#!/usr/bin/env bash
set -euo pipefail

# scripts/dev-all.sh - Run a local replacement of CI tasks (tests, Docker builds, packaging)
# Usage: scripts/dev-all.sh [--skip-docker] [--skip-packages] [--include-ignored]
 
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

SKIP_DOCKER=false
SKIP_PACKAGES=false
INCLUDE_IGNORED=false
ALLOW_NON_WSL=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-docker) SKIP_DOCKER=true; shift ;; 
    --skip-packages) SKIP_PACKAGES=true; shift ;; 
    --include-ignored) INCLUDE_IGNORED=true; shift ;; 
    --allow-non-wsl) ALLOW_NON_WSL=true; shift ;; 
    --help|-h) echo "Usage: $0 [--skip-docker] [--skip-packages] [--include-ignored]"; exit 0 ;; 
    *) echo "Unknown option: $1"; exit 2 ;; 
  esac
done

echo "Running local dev CI tasks from $ROOT_DIR"
# WSL detection & guard
UNAME_S="$(uname -s || echo Unknown)"
IS_WSL=false
if [ -f /proc/version ] && grep -qi microsoft /proc/version; then
  IS_WSL=true
fi

if echo "$UNAME_S" | grep -Eiq 'MINGW|MSYS|CYGWIN|NT|Windows'; then
  if [ "$IS_WSL" = true ]; then
    echo "Detected WSL environment — proceeding."
  else
    echo "Detected a native Windows shell (e.g. Git Bash/Cygwin)."
    echo "This script is designed to run under a POSIX shell or WSL2 for Windows users."
    echo "WSL2 is recommended for parity with CI and packaging."
    if [ "$ALLOW_NON_WSL" = true ]; then
      echo "--allow-non-wsl specified: continuing despite native shell detection."
    else
      echo "Aborting. To continue anyway, re-run with: --allow-non-wsl"
      exit 2
    fi
  fi
fi
cd "$ROOT_DIR"

echo "1) Running host unit & integration tests (single-threaded)"
export RUST_TEST_THREADS=1
if [ "$INCLUDE_IGNORED" = true ]; then
  cargo test --workspace -- --ignored --nocapture
else
  cargo test --workspace -- --nocapture
fi

if [ "$SKIP_DOCKER" = false ]; then
  echo "2) Running Docker-based test suites (Debian/Fedora) using contrib/ci"
  if command -v docker >/dev/null 2>&1; then
    chmod +x ./contrib/ci
    ./contrib/ci docker
  else
    echo "Docker not available - skipping Docker-based tests"
  fi
fi

if [ "$SKIP_PACKAGES" = false ]; then
  echo "3) Building packages via scripts/build-all-packages.sh (Docker-based packaging)"
  if [ -x "./scripts/build-all-packages.sh" ]; then
    if command -v docker >/dev/null 2>&1; then
      chmod +x ./scripts/build-all-packages.sh
      ./scripts/build-all-packages.sh --platforms linux/amd64,linux/arm64 || true
    else
      echo "Docker not available - cannot run packaging via Docker. Try install 'cargo-deb' and 'fpm' for local packaging"
    fi
  else
    echo "scripts/build-all-packages.sh not present or not executable. Skipping packages step"
  fi
fi

echo "Local dev CI tasks completed (skipped docker: $SKIP_DOCKER, skipped packages: $SKIP_PACKAGES, included ignored: $INCLUDE_IGNORED)"
