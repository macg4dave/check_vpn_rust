#!/usr/bin/env bash
set -euo pipefail

if [ "$(uname -s)" != Darwin ]; then
  echo "Package inspection and installation require macOS." >&2
  exit 1
fi
if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <inspect|simulate> <package.pkg>" >&2
  exit 2
fi
mode="$1"
package="$2"
case "$mode" in inspect|simulate) ;; *) echo "Invalid mode: $mode" >&2; exit 2 ;; esac
[ -f "$package" ] || { echo "Package not found: $package" >&2; exit 1; }
test_dir="$(mktemp -d)"
mount_point="$test_dir/volume"
attached=0
cleanup() {
  if [ "$attached" -eq 1 ]; then
    if ! hdiutil detach "$mount_point" >/dev/null; then
      echo "Could not detach $mount_point; leaving $test_dir intact." >&2
      return 1
    fi
  fi
  rm -rf "$test_dir"
}
trap cleanup EXIT

if [ "$mode" = inspect ]; then
  pkgutil --expand "$package" "$test_dir/expanded"
  test -f "$test_dir/expanded/Scripts/postinstall"
  test -e "$test_dir/expanded/Payload"
  head -n 40 "$test_dir/expanded/Scripts/postinstall"
  echo "Package payload and postinstall script are present."
else
  image="$test_dir/check_vpn_sim.sparseimage"
  hdiutil create -size 100m -fs HFS+ -volname check_vpn_sim -type SPARSE -ov "$image" >/dev/null
  mkdir -p "$mount_point"
  hdiutil attach -readwrite -noautoopen -mountpoint "$mount_point" "$image" >/dev/null
  attached=1
  if [ "$(id -u)" -eq 0 ]; then
    installer -pkg "$package" -target "$mount_point"
  else
    sudo installer -pkg "$package" -target "$mount_point"
  fi
  test -x "$mount_point/usr/local/bin/check_vpn"
  test -f "$mount_point/Library/LaunchDaemons/com.check_vpn.agent.plist"
  echo "Package files verified on the temporary volume."
fi
