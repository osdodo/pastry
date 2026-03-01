#!/bin/bash

set -euo pipefail

APP_NAME="Pastry"
CARGO_TOML="Cargo.toml"
VERSION="$(awk '
  /^\[package\][[:space:]]*$/ { in_package=1; next }
  /^\[/ { if (in_package) exit }
  in_package && /^[[:space:]]*version[[:space:]]*=/ {
    line=$0
    sub(/^[[:space:]]*version[[:space:]]*=[[:space:]]*"/, "", line)
    sub(/".*$/, "", line)
    print line
    exit
  }
' "${CARGO_TOML}")"

if [ -z "${VERSION}" ]; then
  echo "Failed to read version from ${CARGO_TOML}" >&2
  exit 1
fi

normalize_arch() {
  case "$1" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64|x86) echo "x86_64" ;;
    *) echo "$1" ;;
  esac
}

DETECTED_ARCH="$(normalize_arch "$(uname -m)")"
REQUESTED_ARCH="${1:-${TARGET_ARCH:-}}"

if [ -n "${REQUESTED_ARCH}" ]; then
  REQUESTED_ARCH="$(normalize_arch "${REQUESTED_ARCH}")"
  if [ "${REQUESTED_ARCH}" != "${DETECTED_ARCH}" ]; then
    echo "Requested arch (${REQUESTED_ARCH}) does not match runner arch (${DETECTED_ARCH})" >&2
    exit 1
  fi
fi

TARGET_ARCH="${REQUESTED_ARCH:-${DETECTED_ARCH}}"

# Build release version
cargo build --release

# Create app bundle structure
APP_DIR="target/release/${APP_NAME}.app"
ARCHIVE_PATH="target/release/${APP_NAME}-${VERSION}-macos-${TARGET_ARCH}.zip"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

# Clean up old app bundle
rm -rf "${APP_DIR}"
rm -f "${ARCHIVE_PATH}"

# Create directory structure
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

# Copy executable file
cp "target/release/pastry" "${MACOS_DIR}/${APP_NAME}"

# Convert PNG icon to ICNS format
# First create temporary iconset directory
ICONSET_DIR="target/release/AppIcon.iconset"
mkdir -p "${ICONSET_DIR}"

# Use sips command to generate icons of different sizes
for size in 16 32 128 256 512; do
  sips -z $size $size assets/logo.png --out "${ICONSET_DIR}/icon_${size}x${size}.png"
  sips -z $((size * 2)) $((size * 2)) assets/logo.png --out "${ICONSET_DIR}/icon_${size}x${size}@2x.png"
done

# Convert to icns format
iconutil -c icns "${ICONSET_DIR}" -o "${RESOURCES_DIR}/AppIcon.icns"

# Clean up temporary files
rm -rf "${ICONSET_DIR}"

# Create Info.plist
cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>${APP_NAME}</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundleIdentifier</key>
  <string>com.pastry.app</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>10.13</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>LSUIElement</key>
  <false/>
</dict>
</plist>
EOF

echo "App bundle created: ${APP_DIR}"
ditto -c -k --sequesterRsrc --keepParent "${APP_DIR}" "${ARCHIVE_PATH}"
echo "Archive created: ${ARCHIVE_PATH}"
echo "Architecture: ${TARGET_ARCH}"
echo "You can move it to /Applications directory or double-click to run"
