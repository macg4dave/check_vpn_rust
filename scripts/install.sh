#!/usr/bin/env bash
set -euo pipefail

# Enhanced installer for check_vpn
# - Interactive prompts for user vs system install
# - Supports Linux systemd (system or user) and macOS launchd (system or user)
# - Uses files from repo by default; supports --binary --config --service
# - Non-interactive: --mode system|user|auto and --yes to accept defaults

BINARY="target/release/check_vpn"
CONFIG_SRC="./check_vpn.xml"
SERVICE_SRC="./check_vpn.service"
PLIST_SRC="../contrib/check_vpn.plist"

MODE="interactive" # interactive | system | user | auto
ASSUME_YES=0
NO_START=0
DRY_RUN=0

usage() {
  cat <<EOF
Usage: $0 [options]

Options:
  --binary <path>    Path to built binary (default: target/release/check_vpn)
  --config <path>    Path to config xml (default: ./check_vpn.xml)
  --service <path>   Path to systemd service file (default: ./check_vpn.service)
  --mode <mode>      Install mode: interactive|system|user|auto
  --yes              Accept prompts (non-interactive)
  --no-start         Install but do not start/enable the service
  -h, --help         Show this help

Examples:
  sudo $0 --mode system           # system-wide install on Linux
  $0 --mode user --yes            # non-root per-user install
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --binary)
      shift; BINARY="$1"; shift;;
    --config)
      shift; CONFIG_SRC="$1"; shift;;
    --service)
      shift; SERVICE_SRC="$1"; shift;;
    --mode)
      shift; MODE="$1"; shift;;
    --yes)
      ASSUME_YES=1; shift;;
    --no-start)
      NO_START=1; shift;;
    --dry-run)
      DRY_RUN=1; shift;;
    -h|--help)
      usage; exit 0;;
    *)
      echo "Unknown arg: $1"; usage; exit 1;;
  esac
done

log() { echo "[install] $*"; }
err() { echo "[install][ERROR] $*" >&2; }

# Run a command or print it if dry-run
run_cmd() {
  if [ "$DRY_RUN" -eq 1 ]; then
    echo "DRY-RUN: $*"
  else
    "$@"
  fi
}

# small helper for y/n prompt
confirm() {
  if [ "$ASSUME_YES" -eq 1 ]; then
    return 0
  fi
  local prompt="$1"
  local default=${2:-}
  local reply
  while true; do
    if [ -n "$default" ]; then
      read -r -p "$prompt [$default] " reply
      reply=${reply:-$default}
    else
      read -r -p "$prompt [y/n] " reply
    fi
    case "${reply,,}" in
      y|yes) return 0;;
      n|no) return 1;;
      *) echo "Please answer yes or no.";;
    esac
  done
}

# Detect OS
OS=$(uname -s)
case "$OS" in
  Linux) PLATFORM=linux ;;
  Darwin) PLATFORM=macos ;;
  *) PLATFORM=unknown ;;
esac

if [ ! -f "$BINARY" ]; then
  err "Binary not found at $BINARY. Try running 'cargo build --release' first."
  exit 1
fi

# Determine install mode interactively if requested
if [ "$MODE" = "interactive" ]; then
  log "Detected platform: $PLATFORM"
  if [ "$PLATFORM" = "macos" ]; then
    echo "Install for macOS (launchd). Choose installation type:"
  echo "  1) System (requires sudo, installs to /usr/local/bin and /Library/LaunchDaemons)"
    echo "  2) User (installs to ~/.local/bin or ~/bin and ~/Library/LaunchAgents)"
    read -r -p "Select 1 or 2 (default 2): " choice
    choice=${choice:-2}
    if [ "$choice" = "1" ]; then MODE=system; else MODE=user; fi
  elif [ "$PLATFORM" = "linux" ]; then
    # prefer systemd if available
    if command -v systemctl >/dev/null 2>&1; then
      echo "Install on Linux with systemd. Choose installation type:"
  echo "  1) System (system service, requires sudo)"
      echo "  2) User (systemd --user, per-user install)"
      read -r -p "Select 1 or 2 (default 1): " choice
      choice=${choice:-1}
      if [ "$choice" = "1" ]; then MODE=system; else MODE=user; fi
    else
      echo "No systemd found. Defaulting to user install.";
      MODE=user
    fi
  else
    echo "Unknown platform: $OS. Defaulting to user install.";
    MODE=user
  fi
fi

log "Chosen install mode: $MODE"

# Helper: install binary to given dir
install_binary() {
  local dest_dir="$1"
  run_cmd mkdir -p "$dest_dir"
  local dest="$dest_dir/check_vpn"
  run_cmd cp "$BINARY" "$dest"
  run_cmd chmod 755 "$dest"
  log "Installed binary to $dest"
}

# Helper: install config file to destination
install_config() {
  local dest_dir="$1"
  if [ -f "$CONFIG_SRC" ]; then
    run_cmd mkdir -p "$dest_dir"
    run_cmd cp "$CONFIG_SRC" "$dest_dir/config.xml"
    run_cmd chmod 644 "$dest_dir/config.xml"
    log "Installed config to $dest_dir/config.xml"
  else
    log "No config found at $CONFIG_SRC; skipping config install. You can place a config at $dest_dir/config.xml later."
  fi
}

if [ "$PLATFORM" = "linux" ]; then
  if [ "$MODE" = "system" ]; then
    # system-wide install -> require root
    if [ "$(id -u)" -ne 0 ]; then
      err "System install requires root. Please run with sudo."
      exit 1
    fi
    BIN_DIR="/usr/local/bin"
    install_binary "$BIN_DIR"
    install_config "/etc/check_vpn"

    # service: prefer user-specified, else use contrib/service src if exists
    if [ -f "$SERVICE_SRC" ]; then
      SERVICE_SRC_PATH="$SERVICE_SRC"
    elif [ -f "../contrib/check_vpn.service" ]; then
      SERVICE_SRC_PATH="../contrib/check_vpn.service"
    else
      SERVICE_SRC_PATH=""
    fi

    if [ -n "$SERVICE_SRC_PATH" ]; then
      SERVICE_DEST="/etc/systemd/system/check_vpn.service"
      run_cmd cp "$SERVICE_SRC_PATH" "$SERVICE_DEST"
      run_cmd chmod 644 "$SERVICE_DEST"
      log "Installed systemd unit to $SERVICE_DEST"
      run_cmd systemctl daemon-reload
      if [ "$NO_START" -eq 0 ]; then
        run_cmd systemctl enable --now check_vpn.service
        log "Enabled and started check_vpn.service"
      else
        log "Service installed but not started ( --no-start )."
      fi
    else
      err "No service file found (checked $SERVICE_SRC and ../contrib/check_vpn.service). Skipping unit installation."
    fi

  else
    # user mode on linux
    HOME_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
    install_binary "$HOME_BIN"
    CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/check_vpn"
    install_config "$CONFIG_DIR"

    # user systemd unit
    if command -v systemctl >/dev/null 2>&1; then
      UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
      run_cmd mkdir -p "$UNIT_DIR"
      if [ -f "$SERVICE_SRC" ]; then
        run_cmd cp "$SERVICE_SRC" "$UNIT_DIR/check_vpn.service"
      elif [ -f "../contrib/check_vpn.service" ]; then
        run_cmd cp "../contrib/check_vpn.service" "$UNIT_DIR/check_vpn.service"
      else
        log "No systemd unit file found for user install; skipping.";
      fi
      if [ -f "$UNIT_DIR/check_vpn.service" ]; then
        run_cmd chmod 644 "$UNIT_DIR/check_vpn.service"
        log "Installed user systemd unit to $UNIT_DIR/check_vpn.service"
        run_cmd systemctl --user daemon-reload
        if [ "$NO_START" -eq 0 ]; then
          run_cmd systemctl --user enable --now check_vpn.service
          log "Enabled and started user check_vpn.service"
        else
          log "User service installed but not started ( --no-start )."
        fi
      fi
    else
      log "systemctl not found; skipping user systemd unit installation."
    fi
  fi

elif [ "$PLATFORM" = "macos" ]; then
  if [ "$MODE" = "system" ]; then
    if [ "$(id -u)" -ne 0 ]; then
      err "System install on macOS requires root. Please run with sudo."
      exit 1
    fi
    BIN_DIR="/usr/local/bin"
    install_binary "$BIN_DIR"
    install_config "/etc/check_vpn"

    # plist -> /Library/LaunchDaemons
    if [ -f "../contrib/check_vpn.plist" ]; then
      PLIST_SRC_PATH="../contrib/check_vpn.plist"
    elif [ -f "./check_vpn.plist" ]; then
      PLIST_SRC_PATH="./check_vpn.plist"
    else
      PLIST_SRC_PATH=""
    fi
    if [ -n "$PLIST_SRC_PATH" ]; then
      PLIST_DEST="/Library/LaunchDaemons/com.check_vpn.plist"
      run_cmd cp "$PLIST_SRC_PATH" "$PLIST_DEST"
      run_cmd chmod 644 "$PLIST_DEST"
      log "Installed launchd plist to $PLIST_DEST"
      if [ "$NO_START" -eq 0 ]; then
        run_cmd launchctl bootstrap system "$PLIST_DEST" || run_cmd launchctl load "$PLIST_DEST"
        log "Loaded system launchd plist"
      fi
    else
      err "No plist found to install. Looked for ../contrib/check_vpn.plist and ./check_vpn.plist"
    fi

  else
    # user install on macOS
    HOME_BIN="${XDG_BIN_HOME:-$HOME/.local/bin}"
    # prefer ~/bin if exists in PATH
    if echo ":$PATH:" | grep -q ":$HOME/bin:"; then
      HOME_BIN="$HOME/bin"
    fi
    install_binary "$HOME_BIN"
    CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/check_vpn"
    install_config "$CONFIG_DIR"

    if [ -f "../contrib/check_vpn.plist" ]; then
      PLIST_SRC_PATH="../contrib/check_vpn.plist"
    elif [ -f "./check_vpn.plist" ]; then
      PLIST_SRC_PATH="./check_vpn.plist"
    else
      PLIST_SRC_PATH=""
    fi
    if [ -n "$PLIST_SRC_PATH" ]; then
      LAUNCH_DIR="$HOME/Library/LaunchAgents"
      run_cmd mkdir -p "$LAUNCH_DIR"
      PLIST_DEST="$LAUNCH_DIR/com.check_vpn.plist"
      run_cmd cp "$PLIST_SRC_PATH" "$PLIST_DEST"
      run_cmd chmod 644 "$PLIST_DEST"
      log "Installed user launchd plist to $PLIST_DEST"
      if [ "$NO_START" -eq 0 ]; then
        run_cmd launchctl bootstrap gui/$(id -u) "$PLIST_DEST" || run_cmd launchctl load "$PLIST_DEST"
        log "Loaded user launchd plist"
      fi
    else
      log "No plist found; skipping launchd installation."
    fi
  fi

else
  err "Unsupported platform: $OS"
  exit 1
fi

log "Installation complete."
