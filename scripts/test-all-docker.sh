#!/usr/bin/env bash
# Run tests inside Debian and Fedora Docker images
# Usage: ./scripts/test-all-docker.sh [--include-ignored]

set -euo pipefail

INCLUDE_IGNORED=false
if [ "${1:-}" = "--include-ignored" ]; then
  INCLUDE_IGNORED=true
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker CLI not found in PATH; install Docker Desktop or Docker Engine and ensure it's runnable from this shell." >&2
  echo "On Windows + WSL: enable WSL integration in Docker Desktop and run this script from the WSL distro or from a Windows shell where Docker is available." >&2
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
  "check_vpn_tests_debian:contrib/Dockerfile.debian"
  "check_vpn_tests_fedora:contrib/Dockerfile.fedora"
)

# Make paths robust — ensure we run from the repository root so relative
# Dockerfile paths like `contrib/Dockerfile.debian` resolve correctly when the
# script is executed from another working directory.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"
cd "$repo_root"

for entry in "${images[@]}"; do
  name=${entry%%:*}
  file=${entry#*:}
  printf "\n=== Building image: %s (Dockerfile: %s) ===\n" "$name" "$file"

  # Guard: stop early if the workspace is enormous (BuildKit / daemon build context
  # issues often occur when the repo contains `target/` or other large artifacts).
  if [ "${DOCKER_BUILD_ALLOW_LARGE:-}" != "1" ]; then
    # If the repository has a .dockerignore that excludes 'target/', skip the heavy-size check
    if [ -f .dockerignore ] && grep -qE '^target(/|$)' .dockerignore; then
      echo ".dockerignore contains 'target/' — skipping repo-size check."
    else
      size_mb=$(du -sm . | awk '{print $1}')
    max_mb=20480
    if [ "$size_mb" -ge "$max_mb" ]; then
      echo "ERROR: repository size is ${size_mb}MB (>= ${max_mb}MB)." >&2
      echo "Docker will send your whole repo as the build context which may fail with BuildKit for very large contexts." >&2
      echo "Suggested fixes: add a .dockerignore to exclude 'target/' or other large files, or set DOCKER_BUILD_ALLOW_LARGE=1 to bypass this check." >&2
      exit 1
    fi
  fi
  fi

  docker build -t "$name" -f "$file" .

  echo "=== Running tests in $name ==="
  docker run --rm "$name" sh -c "cargo test"

  if [ "$INCLUDE_IGNORED" = true ]; then
    echo "=== Running ignored tests in $name ==="
    docker run --rm "$name" sh -c "cargo test -- --ignored"
  fi
done

echo "All Docker test runs completed successfully."
