#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

# Basic build deps
apt-get update
apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev ca-certificates curl sudo git

# Install rustup for the vagrant user if cargo missing
if ! su -l vagrant -c 'command -v cargo' >/dev/null 2>&1; then
  echo "Installing rustup for vagrant user..."
  su -l vagrant -c 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y'
fi

# Build project as vagrant user
su -l vagrant -c 'source $HOME/.cargo/env && cd /vagrant && cargo build --release'

# Run the install script as root (non-interactive path will pick the bundled sample config)
if [ -f /vagrant/target/release/check_vpn ]; then
  /vagrant/contrib/install.sh
else
  echo "Build failed: /vagrant/target/release/check_vpn not found" >&2
  exit 1
fi
