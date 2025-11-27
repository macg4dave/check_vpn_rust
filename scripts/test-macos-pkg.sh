#!/usr/bin/env bash
set -euo pipefail

# Test & verification helper for macOS .pkg produced by scripts/build-macos-pkg.sh
# Usage: test-macos-pkg.sh inspect <pkg-file>
#        test-macos-pkg.sh simulate <pkg-file>
# - inspect: expand the package and show/validate expected paths (Scripts/postinstall, Payload)
# - simulate: install into a temporary disk image and inspect that files would be placed correctly

if [ "$(uname -s)" != "Darwin" ]; then
  echo "This script must be run on macOS (Darwin)." >&2
  exit 0
fi

if [ $# -lt 2 ]; then
  echo "Usage: $0 <inspect|simulate> <path-to-pkg>" >&2
  exit 2
fi

MODE=$1
PKG=$2

if [ ! -f "$PKG" ]; then
  echo "Package file not found: $PKG" >&2
  exit 1
fi

case "$MODE" in
  inspect)
    echo "Expanding package: $PKG"
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT
    pkgutil --expand "$PKG" "$TMPDIR/expanded"

    echo "Contents:"
    find "$TMPDIR/expanded" -maxdepth 3 -print

    if [ -f "$TMPDIR/expanded/Scripts/postinstall" ]; then
      echo "Found postinstall — showing first lines:" 
      head -n 40 "$TMPDIR/expanded/Scripts/postinstall" || true
    else
      echo "Warning: Scripts/postinstall missing" >&2
    fi

    if [ -d "$TMPDIR/expanded/Payload" ] || [ -f "$TMPDIR/expanded/Payload" ]; then
      echo "Payload present (payload extraction may be an archive)."
    else
      echo "Warning: Payload not found in expanded package" >&2
    fi

    echo "Inspection complete."
    ;;

  simulate)
    echo "Simulating install of $PKG into a temporary disk image (no host changes)"
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT
    IMAGE="$TMPDIR/check_vpn_sim.dmg"
    VOLUME_NAME="check_vpn_sim"

    echo "Creating sparse image..."
    hdiutil create -size 100m -fs HFS+ -volname "$VOLUME_NAME" -type SPARSE -ov "$IMAGE" >/dev/null
    MOUNT_POINT=$(hdiutil attach -readwrite -noverify -noautoopen "$IMAGE" | awk '/Volumes/{print $3; exit}')
    if [ -z "$MOUNT_POINT" ]; then
      echo "Failed to mount image" >&2
      exit 1
    fi

    echo "Mounted at $MOUNT_POINT"

    echo "Running installer to target the mounted image (installer may require sudo)"
    # Installer writes as root — run with sudo if not root
    if [ $(id -u) -ne 0 ]; then
      echo "Running installer with sudo to ensure proper ownership/permissions (you may be asked for a password)"
      sudo installer -pkg "$PKG" -target "$MOUNT_POINT"
    else
      installer -pkg "$PKG" -target "$MOUNT_POINT"
    fi

    echo "Install finished. Inspecting contents in simulated image:"
    echo "List /usr/local/bin:" || true
    ls -la "$MOUNT_POINT/usr/local/bin" || true

    echo "List /Library/LaunchDaemons:" || true
    ls -la "$MOUNT_POINT/Library/LaunchDaemons" || true

    echo "Check files exist and look reasonable"
    if [ -f "$MOUNT_POINT/usr/local/bin/check_vpn" ]; then
      echo "/usr/local/bin/check_vpn exists"
      stat -f "%N: mode=%A uname=%Su" "$MOUNT_POINT/usr/local/bin/check_vpn" || true
    else
      echo "ERROR: /usr/local/bin/check_vpn missing in simulated install" >&2
    fi

    if [ -f "$MOUNT_POINT/Library/LaunchDaemons/com.check_vpn.agent.plist" ]; then
      echo "LaunchDaemon plist exists"
      stat -f "%N: mode=%A uname=%Su" "$MOUNT_POINT/Library/LaunchDaemons/com.check_vpn.agent.plist" || true
    else
      echo "ERROR: LaunchDaemon plist missing in simulated install" >&2
    fi

    echo "Unmounting and cleaning up"
    hdiutil detach "$MOUNT_POINT" >/dev/null || true
    rm -f "$IMAGE" || true

    echo "Simulation complete — no changes were made to the host." 
    ;;

  *)
    echo "Invalid mode: $MODE" >&2
    exit 2
    ;;
esac

exit 0
