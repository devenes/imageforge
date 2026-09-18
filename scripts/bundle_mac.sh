#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# bundle_mac.sh - Bundle native dynamic libraries into ImageForge.app
# ==============================================================================
# Inspects ImageForge.app, extracts non-system dylibs (such as libheif, x265,
# libde265, aom, vmaf, sharpyuv), copies them to Contents/Frameworks/, rewrites
# dynamic references to use @rpath, and strictly verifies that no build-machine
# Homebrew paths remain.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 1. Locate ImageForge.app
APP_PATH="${1:-}"

if [[ -z "$APP_PATH" ]]; then
  if [[ -d "$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app" ]]; then
    APP_PATH="$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
  elif [[ -d "$ROOT_DIR/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app" ]]; then
    APP_PATH="$ROOT_DIR/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
  fi
fi

if [[ -z "$APP_PATH" || ! -d "$APP_PATH" ]]; then
  echo "Error: ImageForge.app not found. Build the application first." >&2
  exit 1
fi

echo "==> Bundling native dependencies for: $APP_PATH"

MACOS_DIR="$APP_PATH/Contents/MacOS"
FRAMEWORKS_DIR="$APP_PATH/Contents/Frameworks"

if [[ ! -d "$MACOS_DIR" ]]; then
  echo "Error: MacOS directory not found at $MACOS_DIR" >&2
  exit 1
fi

MAIN_EXEC="$(find "$MACOS_DIR" -type f -perm +111 | head -n 1)"
if [[ -z "$MAIN_EXEC" || ! -f "$MAIN_EXEC" ]]; then
  echo "Error: No executable binary found in $MACOS_DIR" >&2
  exit 1
fi

echo "    Main executable: $MAIN_EXEC"
mkdir -p "$FRAMEWORKS_DIR"

# 2. Ensure @executable_path/../Frameworks is in the main executable's RPATH
if ! otool -l "$MAIN_EXEC" | grep -A2 LC_RPATH | grep -q "@executable_path/../Frameworks"; then
  echo "    Adding @executable_path/../Frameworks to LC_RPATH of main executable..."
  install_name_tool -add_rpath "@executable_path/../Frameworks" "$MAIN_EXEC"
fi

# Function: check if library path is a system library or already relocated
is_system_lib() {
  local lib="$1"
  case "$lib" in
    /System/Library/*) return 0 ;;
    /usr/lib/*) return 0 ;;
    @rpath/*) return 0 ;;
    @loader_path/*) return 0 ;;
    @executable_path/*) return 0 ;;
    *) return 1 ;;
  esac
}

# 3. Recursive inspection & bundling
# We maintain a queue of binaries/dylibs to inspect
declare -a WORKLIST=("$MAIN_EXEC")
QUEUE_INDEX=0
PROCESSED_LIST=""

while [[ $QUEUE_INDEX -lt ${#WORKLIST[@]} ]]; do
  CURRENT_FILE="${WORKLIST[$QUEUE_INDEX]}"
  QUEUE_INDEX=$((QUEUE_INDEX + 1))

  # Check if already processed
  case "$PROCESSED_LIST" in
    *"|$CURRENT_FILE|"*) continue ;;
  esac
  PROCESSED_LIST="${PROCESSED_LIST}|$CURRENT_FILE|"

  echo "    Analyzing dependencies for: $(basename "$CURRENT_FILE")"

  # Extract dependencies from otool -L
  # otool -L output lines: "\t/path/to/lib.dylib (compatibility version...)"
  while IFS= read -r line; do
    # Strip leading whitespace and trailing info
    DEP_PATH=$(echo "$line" | sed -E 's/^[[:space:]]+//; s/ \(compatibility.*//')

    if [[ -z "$DEP_PATH" ]]; then
      continue
    fi

    # Ignore header line showing current file name
    if [[ "$DEP_PATH" == *":" ]]; then
      continue
    fi

    # Ignore dylib self-id
    if [[ "$(basename "$DEP_PATH")" == "$(basename "$CURRENT_FILE")" ]]; then
      continue
    fi

    if is_system_lib "$DEP_PATH"; then
      continue
    fi

    # This is a non-system external dependency (e.g. Homebrew dylib)
    DYLIB_NAME="$(basename "$DEP_PATH")"
    DEST_DYLIB="$FRAMEWORKS_DIR/$DYLIB_NAME"

    # If the dylib is not yet in Frameworks, copy and configure it
    if [[ ! -f "$DEST_DYLIB" ]]; then
      # Resolve real path if DEP_PATH is a symlink
      REAL_SOURCE="$(python3 -c "import os, sys; print(os.path.realpath(sys.argv[1]))" "$DEP_PATH" 2>/dev/null || echo "$DEP_PATH")"
      if [[ ! -f "$REAL_SOURCE" ]]; then
        REAL_SOURCE="$DEP_PATH"
      fi

      if [[ ! -f "$REAL_SOURCE" ]]; then
        echo "Error: Dependent library not found on system: $DEP_PATH" >&2
        exit 1
      fi

      echo "      -> Bundling: $DYLIB_NAME"
      cp -L "$REAL_SOURCE" "$DEST_DYLIB"
      chmod 755 "$DEST_DYLIB"

      # Set dylib ID to @rpath/$DYLIB_NAME
      install_name_tool -id "@rpath/$DYLIB_NAME" "$DEST_DYLIB"

      # Add @loader_path to dylib's LC_RPATH so it can locate sibling dylibs
      if ! otool -l "$DEST_DYLIB" | grep -A2 LC_RPATH | grep -q "@loader_path"; then
        install_name_tool -add_rpath "@loader_path" "$DEST_DYLIB" 2>/dev/null || true
      fi

      # Queue newly copied dylib for recursive analysis
      WORKLIST+=("$DEST_DYLIB")
    fi

    # Rewrite dependency reference in CURRENT_FILE
    install_name_tool -change "$DEP_PATH" "@rpath/$DYLIB_NAME" "$CURRENT_FILE"

  done < <(otool -L "$CURRENT_FILE")
done

# 4. Cross-link review: ensure all dylibs in Frameworks reference each other via @rpath
for dylib in "$FRAMEWORKS_DIR"/*.dylib; do
  [[ -f "$dylib" ]] || continue

  while IFS= read -r line; do
    DEP_PATH=$(echo "$line" | sed -E 's/^[[:space:]]+//; s/ \(compatibility.*//')
    [[ -z "$DEP_PATH" || "$DEP_PATH" == *":" ]] && continue

    DYLIB_NAME="$(basename "$DEP_PATH")"
    if [[ -f "$FRAMEWORKS_DIR/$DYLIB_NAME" && "$DEP_PATH" != "@rpath/$DYLIB_NAME" && "$DYLIB_NAME" != "$(basename "$dylib")" ]]; then
      install_name_tool -change "$DEP_PATH" "@rpath/$DYLIB_NAME" "$dylib"
    fi
  done < <(otool -L "$dylib")
done

# 5. Ad-hoc signature refresh (required runtime reason: on Apple Silicon,
# install_name_tool invalidates Mach-O page hashes, resulting in SIGKILL
# unless ad-hoc page hashes are refreshed via codesign -f -s -).
# This requires NO Apple ID, NO certificate, and NO private keys.
echo "==> Refreshing ad-hoc signatures for modified ARM64 Mach-O files..."
for dylib in "$FRAMEWORKS_DIR"/*.dylib; do
  [[ -f "$dylib" ]] && codesign -f -s - "$dylib" >/dev/null 2>&1 || true
done
codesign -f -s - "$MAIN_EXEC" >/dev/null 2>&1 || true

# 6. Strict verification
echo "==> Verifying bundled binaries and libraries..."
ALL_BINARIES=("$MAIN_EXEC")
for dylib in "$FRAMEWORKS_DIR"/*.dylib; do
  [[ -f "$dylib" ]] && ALL_BINARIES+=("$dylib")
done

FAILED=0
for binary in "${ALL_BINARIES[@]}"; do
  BIN_NAME="$(basename "$binary")"

  # Verify architecture is arm64
  ARCH_INFO=$(file "$binary")
  if ! echo "$ARCH_INFO" | grep -q "arm64"; then
    echo "Error: $BIN_NAME is not arm64 architecture! Details: $ARCH_INFO" >&2
    FAILED=1
  fi

  # Verify no remaining build-machine / Homebrew runtime dependencies
  DEPENDENCIES=$(otool -L "$binary")
  if echo "$DEPENDENCIES" | grep -E "(/opt/homebrew/|/usr/local/opt/|/usr/local/lib/)"; then
    echo "Error: $BIN_NAME still contains forbidden build-machine dependencies:" >&2
    echo "$DEPENDENCIES" | grep -E "(/opt/homebrew/|/usr/local/opt/|/usr/local/lib/)" >&2
    FAILED=1
  fi
done

if [[ $FAILED -ne 0 ]]; then
  echo "Error: Native library bundling failed verification checks!" >&2
  exit 1
fi

echo "==> Successfully bundled $(ls -1 "$FRAMEWORKS_DIR" | wc -l | tr -d ' ') native libraries into Contents/Frameworks/."
for lib in "$FRAMEWORKS_DIR"/*.dylib; do
  [[ -f "$lib" ]] && echo "    • $(basename "$lib")"
done
