#!/usr/bin/env bash
set -euo pipefail

IMAGE_TAG=${IMAGE_TAG:-checkvpn-systemd:latest}
DOCKERFILE=${DOCKERFILE:-contrib/Dockerfile.systemd}
REPO_DIR="$(pwd)"

echo "Building image ${IMAGE_TAG} from ${DOCKERFILE}..."
docker build -t "$IMAGE_TAG" -f "$DOCKERFILE" .

echo "Running container (privileged). Attach an interactive shell to /work (repo mount)."

docker run --privileged \
  --tmpfs /run --tmpfs /run/lock \
  -v /sys/fs/cgroup:/sys/fs/cgroup:ro \
  -v "$REPO_DIR":/work -w /work \
  -it "$IMAGE_TAG"
