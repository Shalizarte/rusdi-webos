#!/bin/bash
# Build script para Rusdi webOS
# Uso: ./build.sh [armv7|aarch64]

set -e

TARGET=${1:-armv7-unknown-linux-gnueabihf}
PROJECT_DIR=$(cd "$(dirname "$0")/.." && pwd)
BUILD_DIR="$PROJECT_DIR/target/$TARGET/release"
PKG_DIR="$PROJECT_DIR/pkg"
APP_DIR="$PKG_DIR/usr/palm/applications/org.xbmc.kodi"

echo "=== Building Rusdi for $TARGET ==="

# Cross-compile
cargo build --release --target=$TARGET

if [ ! -f "$BUILD_DIR/rusdi-webos" ]; then
    echo "ERROR: Binary not found at $BUILD_DIR/rusdi-webos"
    exit 1
fi

echo "=== Creating package structure ==="
rm -rf "$PKG_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$APP_DIR/media"
mkdir -p "$APP_DIR/lib"
mkdir -p "$APP_DIR/system"
mkdir -p "$APP_DIR/addons"

# Copy binary
cp "$BUILD_DIR/rusdi-webos" "$APP_DIR/rusdi-webos"
chmod +x "$APP_DIR/rusdi-webos"

# Copy assets
cp -r "$PROJECT_DIR/assets/media/"* "$APP_DIR/media/"
cp -r "$PROJECT_DIR/assets/lib/"* "$APP_DIR/lib/"
cp -r "$PROJECT_DIR/assets/system/"* "$APP_DIR/system/"
cp -r "$PROJECT_DIR/assets/addons/"* "$APP_DIR/addons/"

# Copy appinfo.json
cp "$PROJECT_DIR/appinfo.json" "$APP_DIR/"

# Copy control
cp "$PROJECT_DIR/packaging/control" "$PKG_DIR/control"

echo "=== Building .ipk with ares-package ==="
cd "$PROJECT_DIR"
ares-package "$PKG_DIR" -o "rusdi-webos_21.0.0_${TARGET//-/_}.ipk"

echo "=== Done! Package: rusdi-webos_21.0.0_${TARGET//-/_}.ipk ==="
