#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/bin" "$work/packages/scripts"
cat > "$work/bin/cargo" <<'CARGO'
#!/usr/bin/env bash
set -eu
if [ "$1" = pkgid ]; then printf '%s\n' 'path+file:///fixture#check_vpn@1.1.0'; fi
CARGO
cat > "$work/bin/docker" <<'DOCKER'
#!/usr/bin/env bash
echo 'Unexpected Docker invocation' >&2
exit 99
DOCKER
cat > "$work/bin/systemctl" <<'SYSTEMCTL'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$SYSTEMCTL_LOG"
SYSTEMCTL
cp "$work/bin/cargo" "$work/bin/cargo-deb"
chmod +x "$work/bin/"*
export PATH="$work/bin:$PATH"

test "$(bash "$repo_root/scripts/package-version.sh")" = 1.1.0
test "$(bash "$repo_root/scripts/package-version.sh" v1.1.0)" = 1.1.0
for invalid in '1.1' 'bad;command' '../outside'; do
  if bash "$repo_root/scripts/package-version.sh" "$invalid" > "$work/output" 2>&1; then
    echo "Invalid version accepted: $invalid" >&2; exit 1
  fi
done
if bash "$repo_root/scripts/docker-build.sh" debian invalid > "$work/output" 2>&1; then
  echo 'Invalid Docker action accepted' >&2; exit 1
fi
if grep -q 'Unexpected Docker invocation' "$work/output"; then exit 1; fi
for script in install.sh build-all-packages.sh; do
  if bash "$repo_root/scripts/$script" --mode > "$work/output" 2>&1; then
    echo "Missing/unknown argument accepted by $script" >&2; exit 1
  fi
done

# A successful build command without a package must never report success.
cp "$repo_root/scripts/"{build-all-packages,package-version}.sh "$work/packages/scripts/"
if bash "$work/packages/scripts/build-all-packages.sh" --no-docker --skip-fedora \
    --version 1.1.0 --output-dir "$work/output with spaces" > "$work/output" 2>&1; then
  echo 'Packaging succeeded without a .deb' >&2; exit 1
fi
grep -q 'No .deb artifacts found' "$work/output"
if bash "$work/packages/scripts/build-all-packages.sh" --version 1.1.0 \
    --platforms 'linux/amd64;unexpected' --output-dir "$work/invalid-platform" > "$work/output" 2>&1; then
  echo 'Unsupported platform accepted' >&2; exit 1
fi
test ! -d "$work/invalid-platform"

# Reinstalling must preserve configuration; systemctl is an isolated stub.
cd "$repo_root"
mkdir -p "$work/config/check_vpn"
printf 'existing config\n' > "$work/config/check_vpn/config.xml"
printf '<config/>\n' > "$work/new-config.xml"
export SYSTEMCTL_LOG="$work/systemctl.log"
XDG_CONFIG_HOME="$work/config" XDG_BIN_HOME="$work/user-bin" \
  bash scripts/install.sh --mode user --yes --no-start --binary "$work/bin/cargo" \
  --config "$work/new-config.xml" > "$work/output" 2>&1
test "$(cat "$work/config/check_vpn/config.xml")" = 'existing config'
if grep -q enable "$SYSTEMCTL_LOG"; then exit 1; fi

# Generate the macOS lifecycle script with fake build tools, then verify that
# installing onto another volume cannot change ownership on the host volume.
mkdir -p "$work/mac/scripts" "$work/mac/contrib" "$work/mac/target/release"
cp "$repo_root/scripts/"{build-macos-pkg,package-version}.sh "$work/mac/scripts/"
cp "$repo_root/contrib/check_vpn.plist" "$work/mac/contrib/"
cp "$work/bin/cargo" "$work/mac/target/release/check_vpn"
cat > "$work/bin/uname" <<'UNAME'
#!/usr/bin/env bash
if [ "${1:-}" = -s ]; then echo Darwin; else /usr/bin/uname "$@"; fi
UNAME
cat > "$work/bin/pkgbuild" <<'PKGBUILD'
#!/usr/bin/env bash
set -eu
while [ "$#" -gt 0 ]; do
  if [ "$1" = --scripts ]; then
    cp "$2/postinstall" "$CAPTURE_POSTINSTALL"
    shift 2
  else
    if [ "$#" -eq 1 ]; then touch "$1"; fi
    shift
  fi
done
PKGBUILD
cat > "$work/bin/chown" <<'CHOWN'
#!/usr/bin/env bash
printf '%s\n' "$@" >> "$CHOWN_LOG"
CHOWN
chmod +x "$work/bin/"*
export CAPTURE_POSTINSTALL="$work/postinstall" CHOWN_LOG="$work/chown.log"
CHECK_VPN_PKG_AUTOLOAD=true bash "$work/mac/scripts/build-macos-pkg.sh" > "$work/output" 2>&1
shellcheck "$CAPTURE_POSTINSTALL"
volume="$work/target volume"
mkdir -p "$volume/usr/local/bin" "$volume/Library/LaunchDaemons"
cp "$work/bin/cargo" "$volume/usr/local/bin/check_vpn"
cp "$repo_root/contrib/check_vpn.plist" "$volume/Library/LaunchDaemons/com.check_vpn.agent.plist"
sh "$CAPTURE_POSTINSTALL" ignored ignored "$volume" > "$work/output" 2>&1
grep -Fq "$volume/usr/local/bin/check_vpn" "$CHOWN_LOG"
if grep -q '^/usr/local/bin/check_vpn$' "$CHOWN_LOG"; then exit 1; fi

# Scan the index, handle spaces, and never echo detected key contents.
git init -q "$work/index"
cd "$work/index"
printf 'ordinary content\n' > 'file with spaces'
git add -- 'file with spaces'
bash "$repo_root/scripts/check-secrets.sh"
printf '%s%s\n' 'BEGIN RSA ' 'PRIVATE KEY' > 'file with spaces'
git add -- 'file with spaces'
if bash "$repo_root/scripts/check-secrets.sh" > "$work/output" 2>&1; then
  echo 'Staged credential marker was not detected' >&2; exit 1
fi
grep -q 'file with spaces' "$work/output"
if grep -q 'BEGIN RSA' "$work/output"; then exit 1; fi

echo 'Shell regression checks passed.'
