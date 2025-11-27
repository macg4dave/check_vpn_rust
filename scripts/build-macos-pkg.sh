#!/usr/bin/env bash
set -euo pipefail

# Local-only macOS system-level package builder for check_vpn.
# Produces: release/macos/check_vpn-<version>.pkg and artifacts/macos/check_vpn-macos.tar.gz
# Default behavior: do NOT auto-load the LaunchDaemon. Set CHECK_VPN_PKG_AUTOLOAD=true when building
# if you want the package's postinstall script to load the daemon automatically on install.

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script must be run on macOS (Darwin)."
  exit 0
fi

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT_PKG_DIR="$ROOT_DIR/release/macos"
ARTIFACT_DIR="$ROOT_DIR/artifacts/macos"
PKGROOT="$(mktemp -d)"
SCRIPTS_DIR="$(mktemp -d)"

VERSION=${VERSION:-}
if [ -z "$VERSION" ]; then
  if git rev-parse --git-dir >/dev/null 2>&1; then
    VERSION=$(git describe --tags --always 2>/dev/null || true)
  fi
fi
if [ -z "$VERSION" ]; then
  VERSION=$(date -u +%Y%m%d%H%M%S)
fi

echo "Building check_vpn macOS package — version: ${VERSION}"

mkdir -p "$OUT_PKG_DIR"
mkdir -p "$ARTIFACT_DIR"

echo "Running cargo build --release"
cargo build --release

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

echo "Setting ownership and permissions for check_vpn components"
if [ -f "/usr/local/bin/check_vpn" ]; then
  chown root:wheel /usr/local/bin/check_vpn || true
  chmod 755 /usr/local/bin/check_vpn || true
fi
if [ -f "/Library/LaunchDaemons/com.check_vpn.agent.plist" ]; then
  chown root:wheel /Library/LaunchDaemons/com.check_vpn.agent.plist || true
  chmod 644 /Library/LaunchDaemons/com.check_vpn.agent.plist || true
fi

echo "Package installation finished. To enable the agent run: sudo launchctl load /Library/LaunchDaemons/com.check_vpn.agent.plist"
exit 0
POST

chmod +x "$POSTINSTALL"

# Optionally include auto-load code if build-time env var was set to true
if [ "${CHECK_VPN_PKG_AUTOLOAD:-false}" = "true" ]; then
  cat >> "$POSTINSTALL" <<'AUTO'

# Auto-load launched at install time
if [ -f "/Library/LaunchDaemons/com.check_vpn.agent.plist" ]; then
  echo "Attempting to load LaunchDaemon (auto-load enabled during build)"
  /bin/launchctl bootstrap system /Library/LaunchDaemons/com.check_vpn.agent.plist >/dev/null 2>&1 || true
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

# Cleanup
rm -rf "$PKGROOT" "$SCRIPTS_DIR"

echo "Done."
