#!/usr/bin/env bash
set -euo pipefail

# Per-user installer for check_vpn.
# Installs binary, per-user config (~/.config/check_vpn/config.xml) and a
# systemd user unit (~/.config/systemd/user/check_vpn.service). Does not require root.

BINARY=${BINARY:-target/release/check_vpn}
DEST_BIN=${DEST_BIN:-"${HOME}/.local/bin/check_vpn"}
CONFIG_SRC=${CONFIG_SRC:-contrib/config.xml}
CONFIG_DEST=${CONFIG_DEST:-"${HOME}/.config/check_vpn/config.xml"}
SYSTEMD_USER_DIR=${SYSTEMD_USER_DIR:-"${HOME}/.config/systemd/user"}
SYSTEMD_UNIT_DEST="${SYSTEMD_USER_DIR}/check_vpn.service"

if [ ! -f "$BINARY" ]; then
  echo "Binary $BINARY not found. Build with: cargo build --release" >&2
  exit 1
fi

mkdir -p "$(dirname "$DEST_BIN")"
mkdir -p "$(dirname "$CONFIG_DEST")"
mkdir -p "$SYSTEMD_USER_DIR"

echo "Installing binary to $DEST_BIN"
install -m 0755 "$BINARY" "$DEST_BIN"

if [ ! -f "$CONFIG_DEST" ]; then
  echo "Installing default config to $CONFIG_DEST"
  install -m 0644 "$CONFIG_SRC" "$CONFIG_DEST"
else
  echo "$CONFIG_DEST already exists; leaving it in place"
fi

# Create a simple systemd --user unit that points to the per-user config
cat > "$SYSTEMD_UNIT_DEST" <<'UNIT'
[Unit]
Description=check_vpn (per-user)
After=network-online.target

[Service]
Type=simple
ExecStart=${HOME}/.local/bin/check_vpn --config-file ${HOME}/.config/check_vpn/config.xml --enable-metrics --metrics-addr 127.0.0.1:9090
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
UNIT

# Ensure the systemd user daemon picks up the unit
systemctl --user daemon-reload || true
systemctl --user enable --now check_vpn.service || echo "Failed to enable user service; you may need to run: systemctl --user enable --now check_vpn.service"

cat <<EOF
Per-user install complete.
Binary: $DEST_BIN
Config: $CONFIG_DEST
Systemd user unit: $SYSTEMD_UNIT_DEST
To view logs: journalctl --user -u check_vpn -f
EOF
