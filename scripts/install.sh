#!/usr/bin/env bash
set -euo pipefail

# install.sh - install the check_vpn Rust binary and systemd unit
# Usage: sudo ./scripts/install.sh [--binary <path-to-binary>] [--config <path-to-config>]

BINARY="target/release/check_vpn"
CONFIG_SRC="./check_vpn.xml"
SERVICE_SRC="./check_vpn.service"
INSTALL_DIR="/usr/local/bin/check_vpn"
SERVICE_DEST="/etc/systemd/system/check_vpn.service"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --binary)
      shift; BINARY="$1"; shift;;
    --config)
      shift; CONFIG_SRC="$1"; shift;;
    --service)
      shift; SERVICE_SRC="$1"; shift;;
    *)
      echo "Unknown arg: $1"; exit 1;;
  esac
done

if [ "$(id -u)" -ne 0 ]; then
  echo "This installer must be run as root (sudo)" >&2
  exit 1
fi

if [ ! -f "$BINARY" ]; then
  echo "Binary not found at $BINARY. Try running 'cargo build --release' first." >&2
  exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/check_vpn"
chmod 755 "$INSTALL_DIR/check_vpn"

# Install config
if [ -f "$CONFIG_SRC" ]; then
  mkdir -p /etc/check_vpn
  cp "$CONFIG_SRC" /etc/check_vpn/config.xml
  chmod 644 /etc/check_vpn/config.xml
  echo "Installed config to /etc/check_vpn/config.xml"
else
  echo "No config file found at $CONFIG_SRC; using defaults. You can place a config at /etc/check_vpn/config.xml later." >&2
fi

# Install systemd unit
if [ -f "$SERVICE_SRC" ]; then
  cp "$SERVICE_SRC" "$SERVICE_DEST"
  chmod 644 "$SERVICE_DEST"
  echo "Installed systemd unit to $SERVICE_DEST"
  systemctl daemon-reload
  systemctl enable --now check_vpn.service
  echo "Enabled and started check_vpn.service"
else
  echo "Service file not found at $SERVICE_SRC; please install a systemd unit manually." >&2
fi

echo "Installation complete."
