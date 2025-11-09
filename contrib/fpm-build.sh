#!/usr/bin/env bash
set -euo pipefail

# Build an RPM using fpm. Requires fpm to be installed (gem install fpm).
# Usage: sudo ./contrib/fpm-build.sh [version]

VERSION=${1:-0.1.0}
PKG_NAME=check_vpn
BUILD_DIR=$(mktemp -d)
DESTDIR=$BUILD_DIR/root

echo "Preparing layout in $DESTDIR"
mkdir -p "$DESTDIR"/usr/bin
mkdir -p "$DESTDIR"/etc/check_vpn
mkdir -p "$DESTDIR"/lib/systemd/system

echo "Building release"
cargo build --release

echo "Copying files"
cp target/release/check_vpn "$DESTDIR"/usr/bin/
cp contrib/config.xml "$DESTDIR"/etc/check_vpn/config.xml
cp contrib/check_vpn.service "$DESTDIR"/lib/systemd/system/check_vpn.service

if ! command -v fpm >/dev/null 2>&1; then
  echo "fpm not found. Install with: sudo gem install --no-document fpm" >&2
  exit 1
fi

echo "Making maintainer scripts executable"
chmod +x contrib/packaging/postinst || true
chmod +x contrib/packaging/prerm || true

echo "Creating RPM via fpm (includes postinst/prerm scripts)"
fpm -s dir -t rpm \
  -n "$PKG_NAME" -v "$VERSION" --iteration 1 \
  --description "VPN monitoring and auto-reconnect" \
  --after-install contrib/packaging/postinst \
  --before-remove contrib/packaging/prerm \
  -C "$DESTDIR" \
  usr/bin/check_vpn \
  etc/check_vpn/config.xml \
  lib/systemd/system/check_vpn.service

echo "RPM created in current directory"
rm -rf "$BUILD_DIR"
