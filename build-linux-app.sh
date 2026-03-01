#!/usr/bin/env bash

set -euo pipefail

APP_NAME="Pastry"
BIN_NAME="pastry"
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

DIST_ROOT="target/linux-release"
APP_DIR="${DIST_ROOT}/${APP_NAME}"
BIN_SOURCE="target/release/${BIN_NAME}"
BIN_TARGET="${APP_DIR}/${BIN_NAME}"
ICON_SOURCE="assets/logo.png"
ICON_TARGET="${APP_DIR}/logo.png"
DESKTOP_FILE="${APP_DIR}/${APP_NAME}.desktop"
ARCHIVE_PATH="${DIST_ROOT}/${APP_NAME}-${VERSION}-linux.tar.gz"

echo "Building release executable ..."
cargo build --release

if [ ! -f "${BIN_SOURCE}" ]; then
  echo "Build failed: executable not found at ${BIN_SOURCE}" >&2
  exit 1
fi

if [ ! -f "${ICON_SOURCE}" ]; then
  echo "Icon file not found: ${ICON_SOURCE}" >&2
  exit 1
fi

rm -rf "${APP_DIR}"
rm -f "${ARCHIVE_PATH}"
mkdir -p "${APP_DIR}"

cp "${BIN_SOURCE}" "${BIN_TARGET}"
cp "${ICON_SOURCE}" "${ICON_TARGET}"
chmod +x "${BIN_TARGET}"

cat > "${DESKTOP_FILE}" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=${APP_NAME}
Comment=Pastry clipboard manager and automation tool
Exec=./${BIN_NAME}
Icon=./logo.png
Terminal=false
Categories=Utility;
EOF

tar -czf "${ARCHIVE_PATH}" -C "${DIST_ROOT}" "${APP_NAME}"

echo ""
echo "Done."
echo "Executable: ${BIN_TARGET}"
echo "Icon file : ${ICON_TARGET}"
echo "Desktop file: ${DESKTOP_FILE}"
echo "Archive file: ${ARCHIVE_PATH}"
