 #!/bin/bash
 # Build script para Rusdi webOS
 # Uso: ./build.sh [armv7|aarch64]
 
 set -e
 
 TARGET=${1:-armv7-unknown-linux-gnueabihf}
 PROJECT_DIR=$(cd "$(dirname "$0")/.." && pwd)
 BUILD_DIR="$PROJECT_DIR/target/$TARGET/release"
 PKG_DIR="$PROJECT_DIR/pkg"
 
 echo "=== Building Rusdi for $TARGET ==="
 
 # Cross-compile
 cargo build --release --target=$TARGET
 
 if [ ! -f "$BUILD_DIR/rusdi-webos" ]; then
     echo "ERROR: Binary not found at $BUILD_DIR/rusdi-webos"
     exit 1
 fi
 
 echo "=== Creating package structure ==="
 rm -rf "$PKG_DIR"
 mkdir -p "$PKG_DIR/usr/palm/applications/org.xbmc.kodi"
 
 # Copy binary
 cp "$BUILD_DIR/rusdi-webos" "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/rusdi-webos"
 chmod +x "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/rusdi-webos"
 
 # Copy assets
 cp -r "$PROJECT_DIR/assets/media/"* "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/media/"
 cp -r "$PROJECT_DIR/assets/lib/"* "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/lib/"
 cp -r "$PROJECT_DIR/assets/system/"* "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/system/"
 cp -r "$PROJECT_DIR/assets/addons/"* "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/addons/"
 
 # Copy appinfo.json
 cp "$PROJECT_DIR/appinfo.json" "$PKG_DIR/usr/palm/applications/org.xbmc.kodi/"
 
 # Copy control
 cp "$PROJECT_DIR/packaging/control" "$PKG_DIR/control"
 
 echo "=== Building .ipk with ares-package ==="
 cd "$PROJECT_DIR"
 ares-package "$PKG_DIR" -o "rusdi-webos_21.0.0_${TARGET//-/_}.ipk"
 
 echo "=== Done! Package: rusdi-webos_21.0.0_${TARGET//-/_}.ipk ==="
