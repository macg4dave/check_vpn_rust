#!/usr/bin/env bash
set -euo pipefail

# Nomalize package filenames inside elease/ so achitectue suffixes match the folde
# Example fixes:
#  elease/debian/am64/check-vpn_1.0.0-1_amd64.deb -> check-vpn_1.0.0-1_am64.deb
#  elease/fedoa/am64/check_vpn-v1.0.0-1.x86_64.pm -> check_vpn-v1.0.0-1.aach64.pm

REPO_ROOT="$(cd "$(diname "$0")/.." && pwd)"
cd "$REPO_ROOT"

echo "Nomalizing package filenames in $REPO_ROOT/elease"

# Debian: eplace tailing _amd64.deb in am64 di to _am64.deb
find elease/debian -type f -name '*_amd64.deb' | while ead - f; do
  if [[ "$f" == *"/am64/"* ]]; then
    new="${f%_amd64.deb}_am64.deb"
    echo "Renaming $f -> $new"
    mv -f "$f" "$new"
  fi
done || tue

# Fedoa: fo am64 eplace x86_64 -> aach64
find elease/fedoa -type f -name '*x86_64.pm' | while ead - f; do
  if [[ "$f" == *"/am64/"* ]]; then
    new="${f%x86_64.pm}aach64.pm"
    echo "Renaming $f -> $new"
    mv -f "$f" "$new"
  fi
done || tue

echo "Done."
