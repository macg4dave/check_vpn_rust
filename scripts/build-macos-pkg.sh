#!/usr/bin/env bash
set -euo pipefail

# Local-only macOS system-level package builder for check_vpn.
# Produces: release/macos/check_vpn-<version>.pkg and artifacts/macos/check_vpn-macos.tar.gz
# Default behavior: do NOT auto-load the LaunchDaemon. Set CHECK_VPN_PKG_AUTOLOAD=true when building
# if you want the package's postinstall script to load the daemon automatically on install.

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script must be run on macOS (Darwin)."
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_PKG_DIR="$ROOT_DIR/release/macos"
ARTIFACT_DIR="$ROOT_DIR/artifacts/macos"
PKGROOT="$(mktemp -d)"
SCRIPTS_DIR="$(mktemp -d)"
cleanup() { rm -rf "$PKGROOT" "$SCRIPTS_DIR"; }
trap cleanup EXIT

VERSION="$(bash "$ROOT_DIR/scripts/package-version.sh" "${VERSION:-}")"
cd "$ROOT_DIR"

echo "Building check_vpn macOS package - version: ${VERSION}"

mkdir -p "$OUT_PKG_DIR"
mkdir -p "$ARTIFACT_DIR"

echo "Running cargo build --release --locked"
cargo build --release --locked

BIN_PATH="$ROOT_DIR/target/release/check_vpn"
if [ ! -x "$BIN_PATH" ]; then
  echo "Error: compiled binary not found at $BIN_PATH" >&2
  exit 1
fi

echo "Staging pkgroot in $PKGROOT"
mkdir -p "$PKGROOT/usr/local/bin"
mkdir -p "$PKGROOT/Library/LaunchDaemons"

cp -f "$BIN_PATH" "$PKGROOT/usr/local/bin/check_vpn"

# Use the provided plist as the system LaunchDaemon entry
PLIST_SRC="$ROOT_DIR/contrib/check_vpn.plist"
if [ ! -f "$PLIST_SRC" ]; then
  echo "Error: missing $PLIST_SRC. Expected plist at contrib/check_vpn.plist" >&2
  exit 1
fi
cp -f "$PLIST_SRC" "$PKGROOT/Library/LaunchDaemons/com.check_vpn.agent.plist"

echo "Creating package scripts (postinstall)"
mkdir -p "$SCRIPTS_DIR"

POSTINSTALL="$SCRIPTS_DIR/postinstall"
cat > "$POSTINSTALL" <<'POST'
#!/bin/sh
# Postinstall: set permissions and ownership for installed files.
set -e

# Installer passes the target volume as argument 3. Never modify the host when
# verifying a package against a mounted image.
target_volume="${3:-/}"
binary="$target_volume/usr/local/bin/check_vpn"
plist="$target_volume/Library/LaunchDaemons/com.check_vpn.agent.plist"
if [ -f "$binary" ]; then
  chown root:wheel "$binary"
  chmod 755 "$binary"
fi
if [ -f "$plist" ]; then
  chown root:wheel "$plist"
  chmod 644 "$plist"
fi
echo "Package installed. Configure /etc/check_vpn/config.xml before enabling the LaunchDaemon."

POST

chmod +x "$POSTINSTALL"

# Optionally include auto-load code if build-time env var was set to true
if [ "${CHECK_VPN_PKG_AUTOLOAD:-false}" = "true" ]; then
  cat >> "$POSTINSTALL" <<'AUTO'

# Auto-load launched at install time
if [ "$target_volume" = / ] && [ -f /etc/check_vpn/config.xml ] && [ -f "$plist" ]; then
  echo "Attempting to load LaunchDaemon (auto-load enabled during build)"
  /bin/launchctl bootstrap system "$plist"
fi
AUTO
fi

echo "Building package"
PKG_NAME="check_vpn-${VERSION}.pkg"
PKG_OUT="$OUT_PKG_DIR/$PKG_NAME"

pkgbuild --root "$PKGROOT" \
  --scripts "$SCRIPTS_DIR" \
  --identifier "com.check_vpn.pkg" \
  --version "$VERSION" \
  --ownership recommended \
  "$PKG_OUT"

echo "Creating tarball artifact of binary"
tar -czf "$ARTIFACT_DIR/check_vpn-macos-${VERSION}.tar.gz" -C "$ROOT_DIR/target/release" check_vpn

echo "Package created: $PKG_OUT"
echo "Tarball created: $ARTIFACT_DIR/check_vpn-macos-${VERSION}.tar.gz"

echo "Done."