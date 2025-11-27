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

echo "Building image $IMAGE from $DOCKERFILE..."
  
# Guard: check repo size before sending build context. Docker will tar the
# current directory and send it as the build context — if your workspace (eg
# target/) is huge, this can exceed daemon/BuildKit limits and fail.
# You can bypass this check by setting DOCKER_BUILD_ALLOW_LARGE=1 in the env.
if [ "${DOCKER_BUILD_ALLOW_LARGE:-}" != "1" ]; then
  # If the repository has an explicit .dockerignore that excludes `target/`
  # then the build context should already be small and we can skip the
  # heavy-size check.
  if [ -f .dockerignore ] && grep -qE '^target(/|$)' .dockerignore; then
    echo ".dockerignore contains 'target/' — skipping repo-size check."
  else
    size_mb=$(du -sm . | awk '{print $1}')
  max_mb=20480 # 20 GB
  if [ "$size_mb" -ge "$max_mb" ]; then
    echo "ERROR: repository size is ${size_mb}MB (>= ${max_mb}MB)." >&2
    echo "Docker will send your whole repo as the build context which may fail with BuildKit for very large contexts." >&2
    echo "Suggested fixes: make sure you have a .dockerignore that excludes 'target/' or large files, or set DOCKER_BUILD_ALLOW_LARGE=1 to bypass this check." >&2
    exit 1
  fi
  fi
fi

docker build -t "$IMAGE" -f "$DOCKERFILE" .

HOST_PWD="$(pwd)"
CONTAINER_WD="/usr/src/check_vpn"

case "$ACTION" in
  build)
    docker run --rm -v "$HOST_PWD":"$CONTAINER_WD" -w "$CONTAINER_WD" "$IMAGE" sh -c "cargo build"
    ;;
  build-release)
    docker run --rm -v "$HOST_PWD":"$CONTAINER_WD" -w "$CONTAINER_WD" "$IMAGE" sh -c "cargo build --release"
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
