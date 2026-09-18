#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# build_release.sh - Master Local Build & Release Pipeline for Apple Silicon
# ==============================================================================
# Builds ImageForge for Apple Silicon macOS (aarch64-apple-darwin) locally:
# runs verification, builds the Tauri app, bundles native dylibs into
# Contents/Frameworks/, and packages an unsigned DMG installer.
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

INSTALL_DEPS=0
for arg in "$@"; do
  if [[ "$arg" == "--install-deps" ]]; then
    INSTALL_DEPS=1
  fi
done

echo "========================================================================"
echo " ImageForge — Local macOS ARM64 Build & Release Pipeline"
echo "========================================================================"

# ------------------------------------------------------------------------------
# 1. Validate Environment
# ------------------------------------------------------------------------------
echo "==> [1/8] Validating build environment..."

OS_NAME="$(uname -s)"
if [[ "$OS_NAME" != "Darwin" ]]; then
  echo "Error: ImageForge release builds only run on macOS." >&2
  exit 1
fi

ARCH_NAME="$(uname -m)"
if [[ "$ARCH_NAME" != "arm64" ]]; then
  echo "Error: ImageForge release builds currently require an Apple Silicon Mac." >&2
  exit 1
fi

for cmd in brew cargo rustc node pnpm; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Error: Required tool '$cmd' is not installed or not in PATH." >&2
    exit 1
  fi
done

echo "    • OS: macOS ($OS_NAME)"
echo "    • Architecture: Apple Silicon ($ARCH_NAME)"
echo "    • Rust: $(rustc --version)"
echo "    • Node: $(node --version)"
echo "    • pnpm: $(pnpm --version)"

# ------------------------------------------------------------------------------
# 2. Dependency Setup
# ------------------------------------------------------------------------------
echo "==> [2/8] Checking system native dependencies..."

BREW_PREFIX="$(brew --prefix)"
export PKG_CONFIG_PATH="$BREW_PREFIX/lib/pkgconfig:${PKG_CONFIG_PATH:-}"
export LIBRARY_PATH="$BREW_PREFIX/lib:${LIBRARY_PATH:-}"
export CPATH="$BREW_PREFIX/include:${CPATH:-}"

REQUIRED_BREW_PACKAGES=("jpeg-turbo" "webp" "libheif" "cmake" "nasm")
MISSING_PACKAGES=()

for pkg in "${REQUIRED_BREW_PACKAGES[@]}"; do
  if ! brew list --formula "$pkg" >/dev/null 2>&1; then
    MISSING_PACKAGES+=("$pkg")
  fi
done

if [[ ${#MISSING_PACKAGES[@]} -gt 0 ]]; then
  echo "Warning: The following Homebrew packages are missing: ${MISSING_PACKAGES[*]}"
  if [[ "$INSTALL_DEPS" -eq 1 ]]; then
    echo "    Installing missing packages with Homebrew (--install-deps specified)..."
    brew install "${MISSING_PACKAGES[@]}"
  else
    echo "Error: Missing required build dependencies." >&2
    echo "Please install them by running:" >&2
    echo "    brew install ${MISSING_PACKAGES[*]}" >&2
    echo "Or rerun this script with the --install-deps flag:" >&2
    echo "    $0 --install-deps" >&2
    exit 1
  fi
fi

# Ensure pkg-config or pkgconf is available
if ! command -v pkg-config >/dev/null 2>&1 && ! command -v pkgconf >/dev/null 2>&1; then
  echo "Error: Neither pkg-config nor pkgconf was found. Run: brew install pkgconf" >&2
  exit 1
fi

echo "    • Native dependencies verified with prefix: $BREW_PREFIX"

# ------------------------------------------------------------------------------
# 3. Synchronize / Read Version
# ------------------------------------------------------------------------------
echo "==> [3/8] Resolving project version..."

if [[ -f "$ROOT_DIR/scripts/set-version.mjs" ]]; then
  # Optionally sync from tag if on exact tag
  node "$ROOT_DIR/scripts/set-version.mjs" --from-tag >/dev/null 2>&1 || true
  VERSION="$(node "$ROOT_DIR/scripts/set-version.mjs" --get)"
else
  VERSION="$(grep -m1 '"version"' "$ROOT_DIR/package.json" | tr -d '", ' | cut -d: -f2)"
fi

echo "    • Application version: $VERSION"

# ------------------------------------------------------------------------------
# 4. Install Dependencies and Run Verification Tests
# ------------------------------------------------------------------------------
echo "==> [4/8] Installing JavaScript dependencies and running test suite..."

cd "$ROOT_DIR"
pnpm install --frozen-lockfile

echo "    Building frontend (pnpm build)..."
pnpm build

echo "    Running Rust format check (cargo fmt)..."
cargo fmt --all -- --check

echo "    Running Rust linter (cargo clippy)..."
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

echo "    Running Rust tests (cargo test)..."
cargo test --manifest-path src-tauri/Cargo.toml --all-targets

# ------------------------------------------------------------------------------
# 5. Build Tauri ARM64 Application
# ------------------------------------------------------------------------------
echo "==> [5/8] Building Tauri ARM64 application (aarch64-apple-darwin)..."

# Ensure src-tauri/target symlink resolves to ../target
if [[ ! -e "$ROOT_DIR/src-tauri/target" ]]; then
  ln -sfn ../target "$ROOT_DIR/src-tauri/target"
fi

pnpm tauri build --target aarch64-apple-darwin --bundles app --no-sign

APP_PATH="$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
if [[ ! -d "$APP_PATH" ]]; then
  APP_PATH="$ROOT_DIR/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
fi

if [[ ! -d "$APP_PATH" ]]; then
  echo "Error: Expected ImageForge.app not found after build!" >&2
  exit 1
fi

echo "    • Application built at: $APP_PATH"

# ------------------------------------------------------------------------------
# 6. Bundle Native Dynamic Libraries
# ------------------------------------------------------------------------------
echo "==> [6/8] Bundling native dynamic libraries into application bundle..."

"$ROOT_DIR/scripts/bundle_mac.sh" "$APP_PATH"

# ------------------------------------------------------------------------------
# 7. Create DMG Installer
# ------------------------------------------------------------------------------
echo "==> [7/8] Creating and verifying macOS DMG installer..."

"$ROOT_DIR/scripts/create_dmg.sh" "$APP_PATH"

DMG_PATH="$ROOT_DIR/dist/ImageForge_${VERSION}_aarch64.dmg"
if [[ ! -f "$DMG_PATH" ]]; then
  echo "Error: DMG file was not created at: $DMG_PATH" >&2
  exit 1
fi

DMG_SIZE="$(du -sh "$DMG_PATH" | cut -f1)"

# ------------------------------------------------------------------------------
# 8. Complete & Summary
# ------------------------------------------------------------------------------
echo ""
echo "========================================================================"
echo "ImageForge build complete"
echo "========================================================================"
echo ""
echo "Application:"
echo "src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ImageForge.app"
echo ""
echo "Installer:"
echo "dist/ImageForge_${VERSION}_aarch64.dmg"
echo ""
echo "Architecture: arm64"
echo "Version: $VERSION"
echo "DMG size: $DMG_SIZE"
echo ""
