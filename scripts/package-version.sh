#!/usr/bin/env bash
set -euo pipefail

# Cargo is authoritative; old Git tags must not label a new package.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ "$#" -gt 1 ]; then
  echo "Usage: $0 [X.Y.Z]" >&2
  exit 2
fi
version="${1:-}"
if [ -z "$version" ]; then
  package_id="$(cargo pkgid --offline --locked --manifest-path "$repo_root/Cargo.toml")"
  version="${package_id##*#}"
  version="${version##*@}"
fi
version="${version#v}"
if [[ ! "$version" =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]]; then
  echo "Package version must be a stable X.Y.Z version: $version" >&2
  exit 2
fi
printf '%s\n' "$version"
