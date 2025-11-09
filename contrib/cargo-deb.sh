#!/usr/bin/env bash
set -euo pipefail

# Helper to build a .deb using cargo-deb if installed.
# Install cargo-deb with: cargo install cargo-deb

if ! command -v cargo-deb >/dev/null 2>&1; then
  echo "cargo-deb not found. Install: cargo install cargo-deb" >&2
  exit 1
fi

echo "Building .deb with cargo-deb"
cargo deb --no-strip

echo "Deb(s) created in target/debian/"
