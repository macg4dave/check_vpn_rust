#!/usr/bin/env bash
# Run tests inside all Docker images (base, Debian, Fedora)
# Usage: ./scripts/test-all-docker.sh [--include-ignored]

set -euo pipefail

INCLUDE_IGNORED=false
if [ "${1:-}" = "--include-ignored" ]; then
  INCLUDE_IGNORED=true
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker CLI not found in PATH; install Docker Desktop or Docker Engine and ensure it's runnable from this shell." >&2
  echo "On Windows + WSL: enable WSL integration in Docker Desktop and run this script from the WSL distro or from PowerShell where Docker is available." >&2
  echo "See: https://docs.docker.com/desktop/windows/wsl/" >&2
  exit 1
fi

# Quick operational check: ensure the daemon is responding
if ! docker info >/dev/null 2>&1; then
  echo "Docker CLI found but daemon not responding. Start Docker Desktop (Linux engine) and retry." >&2
  echo "On Windows you may need to start Docker Desktop and enable WSL integration for this distro." >&2
  exit 1
fi

images=(
  "check_vpn_tests:contrib/Dockerfile"
  "check_vpn_tests_debian:contrib/Dockerfile.debian"
  "check_vpn_tests_fedora:contrib/Dockerfile.fedora"
)

for entry in "${images[@]}"; do
  name=${entry%%:*}
  file=${entry#*:}
  echo "\n=== Building image: $name (Dockerfile: $file) ==="
  docker build -t "$name" -f "$file" .

  echo "=== Running tests in $name ==="
  docker run --rm "$name" sh -c "cargo test"

  if [ "$INCLUDE_IGNORED" = true ]; then
    echo "=== Running ignored tests in $name ==="
    docker run --rm "$name" sh -c "cargo test -- --ignored"
  fi
done

echo "All Docker test runs completed successfully."
