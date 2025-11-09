#!/usr/bin/env bash
set -euo pipefail

# Idempotent install script for manual installs / packaging demos.
# Run as root. Copies binary, sample config and systemd unit, creates dirs
# and enables the service. Safe to run multiple times.

PREFIX=${PREFIX:-/usr/local}
BINARY=${BINARY:-target/release/check_vpn}
DEST_BIN=${DEST_BIN:-"${PREFIX}/bin/check_vpn"}
SYSTEMD_UNIT_DEST=${SYSTEMD_UNIT_DEST:-/etc/systemd/system/check_vpn.service}
USER=${USER:-checkvpn}
GROUP=${GROUP:-checkvpn}

if [ "$EUID" -ne 0 ]; then
  echo "This script must be run as root" >&2
  exit 1
fi

if [ ! -f "$BINARY" ]; then
  echo "Binary $BINARY not found. Build with: cargo build --release" >&2
  exit 1
fi

echo "Creating service user/group if missing: $USER"
if ! id -u "$USER" >/dev/null 2>&1; then
  # create system user without home and without login shell
  useradd --system --no-create-home --group nogroup --shell /usr/sbin/nologin "$USER" 2>/dev/null || {
    # fallback if group nogroup doesn't exist
    useradd --system --no-create-home --shell /usr/sbin/nologin "$USER"
  }
fi

echo "Installing binary to $DEST_BIN"
mkdir -p "$(dirname "$DEST_BIN")"
install -m 0755 "$BINARY" "$DEST_BIN"

echo "Installing config to /etc/check_vpn/config.xml (if missing)"
mkdir -p /etc/check_vpn
if [ ! -f /etc/check_vpn/config.xml ]; then
  install -m 0644 contrib/config.xml /etc/check_vpn/config.xml
else
  echo "/etc/check_vpn/config.xml already exists; leaving it in place"
fi

echo "Creating runtime dirs and logdir"
mkdir -p /var/log/check_vpn
chown -R "$USER":"$GROUP" /etc/check_vpn || true
chown -R "$USER":"$GROUP" /var/log/check_vpn || true

echo "Installing systemd unit to $SYSTEMD_UNIT_DEST"
install -m 0644 contrib/check_vpn.service "$SYSTEMD_UNIT_DEST"

echo "Reloading systemd and enabling service"
systemctl daemon-reload
# Try to enable but do not fail install if it can't (packagers may avoid auto-start)
systemctl enable --now check_vpn.service || echo "systemctl enable failed; continuing"

# SELinux: if present, offer to set contexts for /var/log/check_vpn and binary
if command -v selinuxenabled >/dev/null 2>&1 && selinuxenabled; then
  echo "SELinux is enabled on this host. Attempting to set reasonable file contexts."
  if command -v semanage >/dev/null 2>&1; then
    semanage fcontext -a -t var_log_t "/var/log/check_vpn(/.*)?" || true
    restorecon -Rv /var/log/check_vpn || true
    restorecon -v "$DEST_BIN" || true
  else
    echo "semanage not available; to persist contexts consider installing policycoreutils-python-utils or set contexts manually"
  fi
fi

echo "Install complete. Monitor logs with: journalctl -u check_vpn -f"
