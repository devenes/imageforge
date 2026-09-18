#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# create_dmg.sh - Create macOS Drag-to-Applications DMG using native hdiutil
# ==============================================================================
# Packages ImageForge.app into a compressed .dmg with an /Applications symlink,
# verified with hdiutil.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_PATH="${1:-}"

if [[ -z "$APP_PATH" ]]; then
  if [[ -d "$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app" ]]; then
    APP_PATH="$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
  elif [[ -d "$ROOT_DIR/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app" ]]; then
    APP_PATH="$ROOT_DIR/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
  fi
fi

if [[ -z "$APP_PATH" || ! -d "$APP_PATH" ]]; then
  echo "Error: ImageForge.app not found at: $APP_PATH" >&2
  exit 1
fi

# Determine version
if [[ -f "$ROOT_DIR/scripts/set-version.mjs" ]]; then
  VERSION="$(node "$ROOT_DIR/scripts/set-version.mjs" --get)"
else
  VERSION="$(grep -m1 '"version"' "$ROOT_DIR/package.json" | tr -d '", ' | cut -d: -f2)"
fi

if [[ -z "$VERSION" ]]; then
  echo "Error: Could not determine application version." >&2
  exit 1
fi

VOLUME_NAME="ImageForge"
DMG_NAME="ImageForge_${VERSION}_aarch64.dmg"
DIST_DIR="$ROOT_DIR/dist"
OUTPUT_DMG="$DIST_DIR/$DMG_NAME"

mkdir -p "$DIST_DIR"
rm -f "$OUTPUT_DMG"

echo "==> Creating DMG installer for ImageForge v${VERSION}..."
echo "    App bundle: $APP_PATH"
echo "    Target DMG: $OUTPUT_DMG"

# Setup temporary staging directory
STAGING_DIR="/tmp/imageforge-dmg"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"

cleanup() {
  rm -rf "$STAGING_DIR"
}
trap cleanup EXIT INT TERM

echo "    Populating staging directory..."
cp -R "$APP_PATH" "$STAGING_DIR/ImageForge.app"
ln -s /Applications "$STAGING_DIR/Applications"

echo "    Creating compressed UDZO DMG using hdiutil..."
hdiutil create -volname "$VOLUME_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDZO \
  -imagekey zlib-level=9 \
  "$OUTPUT_DMG" -quiet

# Verify DMG integrity
echo "==> Verifying created DMG with hdiutil verify..."
hdiutil verify "$OUTPUT_DMG"

# Mount and verify payload
echo "==> Verifying DMG payload..."
VERIFY_MNT="/tmp/imageforge-verify-$$"
mkdir -p "$VERIFY_MNT"

cleanup_verify() {
  if mount | grep -q "$VERIFY_MNT"; then
    hdiutil detach "$VERIFY_MNT" -force -quiet || true
  fi
  rm -rf "$VERIFY_MNT"
}

hdiutil attach "$OUTPUT_DMG" -mountpoint "$VERIFY_MNT" -noautoopen -nobrowse -quiet

PAYLOAD_APP="$VERIFY_MNT/ImageForge.app"
PAYLOAD_APPLICATIONS="$VERIFY_MNT/Applications"

if [[ ! -d "$PAYLOAD_APP" ]]; then
  echo "Error: ImageForge.app missing inside DMG!" >&2
  cleanup_verify
  exit 1
fi

if [[ ! -L "$PAYLOAD_APPLICATIONS" && ! -e "$PAYLOAD_APPLICATIONS" ]]; then
  echo "Error: Applications link missing inside DMG!" >&2
  cleanup_verify
  exit 1
fi

# Verify payload executable architecture
PAYLOAD_EXEC="$(find "$PAYLOAD_APP/Contents/MacOS" -type f -perm +111 | head -n 1)"
if ! file "$PAYLOAD_EXEC" | grep -q "arm64"; then
  echo "Error: Payload binary inside DMG is not arm64!" >&2
  cleanup_verify
  exit 1
fi

# Verify bundled frameworks
FRAMEWORK_COUNT=$(ls -1 "$PAYLOAD_APP/Contents/Frameworks"/*.dylib 2>/dev/null | wc -l | tr -d ' ')
if [[ "$FRAMEWORK_COUNT" -eq 0 ]]; then
  echo "Error: No bundled frameworks found inside DMG payload!" >&2
  cleanup_verify
  exit 1
fi

cleanup_verify

DMG_SIZE=$(du -sh "$OUTPUT_DMG" | cut -f1)
echo "==> DMG successfully created and verified!"
echo "    Installer: $OUTPUT_DMG"
echo "    Size: $DMG_SIZE"
echo "    Bundled libraries: $FRAMEWORK_COUNT"
