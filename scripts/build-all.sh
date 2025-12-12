#!/usr/bin/env bash
set -euo pipefail

# Minimal POSIX helper to build host + Docker release artifacts (WSL-friendly).
# Runs a local release build (host), and builds Debian/Fedora Docker release artifacts.

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

echo "=== Host release build ==="
cargo build --release

echo "=== Debian Docker release build ==="
if command -v docker >/dev/null 2>&1; then
  ./scripts/docker-build.sh debian build-release
else
  echo "docker not available; skipping debian docker release build"
fi

echo "=== Fedora Docker release build ==="
if command -v docker >/dev/null 2>&1; then
  ./scripts/docker-build.sh fedora build-release
else
  echo "docker not available; skipping fedora docker release build"
fi

echo "Note: building Windows-packaged artifacts is not supported from WSL/posix directly in this helper; use CI or a native Windows build for a packaged .zip."

echo "All builds complete. Check target/release and artifacts/ for outputs."