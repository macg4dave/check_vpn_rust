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
