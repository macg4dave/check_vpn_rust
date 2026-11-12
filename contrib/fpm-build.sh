#!/usr/bin/env bash
set -euo pipefail

# Build an RPM using fpm. Requires fpm to be installed (gem install fpm).
# Usage: ./contrib/fpm-build.sh [version]

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"
VERSION="$(bash "$REPO_ROOT/scripts/package-version.sh" "${1:-}")"
if ! command -v fpm >/dev/null 2>&1; then
  echo "fpm not found. Install with: gem install --no-document fpm" >&2
  exit 1
fi
TARGET_DIR=${CARGO_TARGET_DIR:-target}
PKG_NAME=check_vpn
BUILD_DIR=$(mktemp -d)
DESTDIR="$BUILD_DIR/root"
cleanup() { rm -rf "$BUILD_DIR"; }
trap cleanup EXIT

echo "Preparing layout in $DESTDIR"
mkdir -p "$DESTDIR/usr/share/doc/check_vpn" "$DESTDIR/usr/bin" "$DESTDIR/etc/check_vpn" "$DESTDIR/lib/systemd/system"

echo "Building release (target dir: ${TARGET_DIR})"
if [ -n "${CARGO_TARGET_DIR:-}" ]; then
  CARGO_TARGET_DIR="${TARGET_DIR}" cargo build --release --locked
else
  cargo build --release --locked
fi

echo "Copying files"
cp "${TARGET_DIR}/release/check_vpn" "$DESTDIR/usr/bin/"
cp INSTALL.md "$DESTDIR/usr/share/doc/check_vpn/"
cp contrib/config.xml "$DESTDIR/etc/check_vpn/config.xml"
cp contrib/check_vpn.service "$DESTDIR/lib/systemd/system/check_vpn.service"

mkdir -p "$DESTDIR/lib/systemd/system/check_vpn.service.d" "$DESTDIR/etc/logrotate.d"
cp contrib/systemd/check_vpn.service.d/limits.conf "$DESTDIR/lib/systemd/system/check_vpn.service.d/"
cp contrib/logrotate.conf "$DESTDIR/etc/logrotate.d/check_vpn"

echo "Creating RPM via fpm (includes postinst/prerm scripts)"
fpm -s dir -t rpm \
  -n "$PKG_NAME" -v "$VERSION" --iteration 1 \
  --license MIT \
  --description "VPN monitoring and auto-reconnect" \
  --after-install contrib/packaging/postinst \
  --before-remove contrib/packaging/prerm \
  -C "$DESTDIR" \
  usr/share/doc/check_vpn/INSTALL.md \
  usr/bin/check_vpn \
  etc/check_vpn/config.xml \
  lib/systemd/system/check_vpn.service \
  lib/systemd/system/check_vpn.service.d \
  etc/logrotate.d/check_vpn

echo "RPM created in current directory"