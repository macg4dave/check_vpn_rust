#!/usr/bin/env bash
set -euo pipefail

# Normalize package filenames inside release/ so architecture suffixes match the folder
# Example fixes:
#  release/debian/arm64/check-vpn_1.0.0-1_amd64.deb -> check-vpn_1.0.0-1_arm64.deb
#  release/fedora/arm64/check_vpn-v1.0.0-1.x86_64.rpm -> check_vpn-v1.0.0-1.aarch64.rpm

REPO_ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$REPO_ROOT"

echo "Normalizing package filenames in $REPO_ROOT/release"

# Debian: replace trailing _amd64.deb in arm64 dir to _arm64.deb
find release/debian -type f -name '*_amd64.deb' | while read -r f; do
  if [[ "$f" == *"/arm64/"* ]]; then
    new=$(echo "$f" | sed 's/_amd64\.deb$/_arm64.deb/')
    echo "Renaming $f -> $new"
    mv -f "$f" "$new"
  fi
done || true

# Fedora: for arm64 replace x86_64 -> aarch64
find release/fedora -type f -name '*x86_64.rpm' | while read -r f; do
  if [[ "$f" == *"/arm64/"* ]]; then
    new=$(echo "$f" | sed 's/x86_64\.rpm$/aarch64.rpm/')
    echo "Renaming $f -> $new"
    mv -f "$f" "$new"
  fi
done || true

echo "Done."
