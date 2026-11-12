#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $0 <distro> <action>

distro: debian | fedora
action: build | build-release | test | test-ignored

Examples:
  $0 debian build
  $0 fedora build-release
  $0 debian test
  $0 fedora test-ignored
EOF
}

if [ "$#" -lt 2 ]; then
  usage
  exit 2
fi

DISTRO="$1"
ACTION="$2"

case "$DISTRO" in
  debian)
    DOCKERFILE="contrib/Dockerfile.debian"
    IMAGE="check_vpn_tests_debian"
    ;;
  fedora)
    DOCKERFILE="contrib/Dockerfile.fedora"
    IMAGE="check_vpn_tests_fedora"
    ;;
  *)
    echo "Unsupported distro: $DISTRO"
    usage
    exit 2
    ;;
esac

case "$ACTION" in
  build|build-release|test|test-ignored) ;;
  *) echo "Unsupported action: $ACTION" >&2; usage; exit 2 ;;
esac
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$script_dir/.."

echo "Building image $IMAGE from $DOCKERFILE..."

if [ "${DOCKER_BUILD_ALLOW_LARGE:-}" != "1" ]; then
  if [ -f .dockerignore ] && grep -qE '^target(/|$)' .dockerignore; then
    echo ".dockerignore excludes target/ - skipping repo-size check."
  else
    size_mb=$(du -sm . | awk '{print $1}')
    max_mb=20480 # 20 GB
    if [ "$size_mb" -ge "$max_mb" ]; then
      echo "ERROR: repository size is ${size_mb}MB (>= ${max_mb}MB)." >&2
      echo "Docker will send your whole repo as the build context which may fail with BuildKit for very large contexts." >&2
      echo "Suggested fixes: ensure .dockerignore excludes target/ or large files, or set DOCKER_BUILD_ALLOW_LARGE=1 to bypass this check." >&2
      exit 1
    fi
  fi
fi

docker build -t "$IMAGE" -f "$DOCKERFILE" .

HOST_PWD="$(pwd)"
CONTAINER_WD="/usr/src/check_vpn"

case "$ACTION" in
  build)
    docker run --rm -e "CARGO_TARGET_DIR=$CONTAINER_WD/target-$DISTRO" -v "$HOST_PWD":"$CONTAINER_WD" -w "$CONTAINER_WD" "$IMAGE" cargo build --locked
    ;;
  build-release)
    docker run --rm -e "CARGO_TARGET_DIR=$CONTAINER_WD/target-$DISTRO" -v "$HOST_PWD":"$CONTAINER_WD" -w "$CONTAINER_WD" "$IMAGE" cargo build --release --locked
    ;;
  test)
    docker run --rm "$IMAGE"
    ;;
  test-ignored)
    docker run --rm "$IMAGE" sh -c "cargo test -- --ignored"
    ;;
  *)
    echo "Unsupported action: $ACTION"
    usage
    exit 2
    ;;
esac