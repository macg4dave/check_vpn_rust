#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
if ! command -v docker >/dev/null 2>&1 || ! docker info >/dev/null 2>&1; then
  echo "Host, Debian, and Fedora builds require a working Docker daemon." >&2
  exit 1
fi
cargo build --release --locked
bash scripts/docker-build.sh debian build-release
bash scripts/docker-build.sh fedora build-release
echo "Built host target/release, target-debian/release, and target-fedora/release."
