#!/usr/bin/env bash
set -euo pipefail

# Reproducible packaging helper. Builds Debian (.deb) and Fedora (.rpm) packages
# inside Docker images and places artifacts into release/debian/ and
# release/fedora/ respectively.
#
# Usage:
#  ./scripts/build-all-packages.sh [--version X.Y.Z] [--output-dir ./release] [--no-docker] [--skip-debian] [--skip-fedora]

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
DOCKER_DEBIAN_TAG=check_vpn_debian_packaging
DOCKER_FEDORA_TAG=check_vpn_fedora_packaging
# Buildx cache behavior if BUILD_CACHE=1
BUILD_CACHE=${BUILD_CACHE:-0}
OUTPUT_DIR_DEFAULT="${REPO_ROOT}/release"
PLATFORMS="linux/amd64,linux/arm64"
VERSION=""
OUTPUT_DIR="$OUTPUT_DIR_DEFAULT"
USE_DOCKER=1
SKIP_DEBIAN=0
SKIP_FEDORA=0

show_help() {
  cat <<EOF
Usage: $0 [--version X.Y.Z] [--output-dir DIR] [--no-docker] [--skip-debian] [--skip-fedora]

By default this script builds both Debian (.deb) and Fedora (.rpm) packages in Docker
containers and writes them to ${OUTPUT_DIR_DEFAULT}/debian and ${OUTPUT_DIR_DEFAULT}/fedora.
Use --no-docker to attempt local packaging (requires cargo-deb and fpm installed locally).
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --version|-v)
      VERSION="$2"; shift 2;;
    --output-dir)
      OUTPUT_DIR="$2"; shift 2;;
    --no-docker)
      USE_DOCKER=0; shift;;
    --skip-debian)
      SKIP_DEBIAN=1; shift;;
    --skip-fedora)
      SKIP_FEDORA=1; shift;;
    --platforms)
      PLATFORMS="$2"; shift 2;;
    --help|-h)
      show_help; exit 0;;
    *)
      echo "Unknown argument: $1" >&2; show_help; exit 2;;
  esac
done

if [ -z "${VERSION}" ]; then
  # Try to derive from git tags; fallback to 0.0.0
  if command -v git >/dev/null 2>&1 && [ -d "${REPO_ROOT}/.git" ]; then
    VERSION=$(git -C "${REPO_ROOT}" describe --tags --abbrev=0 2>/dev/null || true)
    VERSION=${VERSION:-0.0.0}
  else
    VERSION=0.0.0
  fi
fi

mkdir -p "$OUTPUT_DIR/debian"
mkdir -p "$OUTPUT_DIR/fedora"
mkdir -p "$OUTPUT_DIR/unix"

# split platform list into array (comma separated)
IFS=',' read -r -a PLATFORM_ARRAY <<< "$PLATFORMS"

echo "Packaging version: ${VERSION}"
echo "Output directory: ${OUTPUT_DIR}"

run_debian() {
  echo "=== Debian packaging (docker=${USE_DOCKER}) ==="
  if [ "$USE_DOCKER" -eq 1 ]; then
    # If buildx is available and multiple platforms specified, use it to build per-platform images
    if command -v docker-buildx >/dev/null 2>&1 || docker buildx version >/dev/null 2>&1; then
      # Ensure a buildx builder is present
      if ! docker buildx inspect > /dev/null 2>&1; then
        docker buildx create --use --name buildx-builder || true
      fi

      for plat in "${PLATFORM_ARRAY[@]}"; do
        arch=$(echo "$plat" | awk -F'/' '{print $2}')
        img_tag="${DOCKER_DEBIAN_TAG}-${arch}"
          echo "Building image for platform ${plat} -> ${img_tag}"
          CACHE_FLAGS=""
          if [ "${BUILD_CACHE}" = "1" ]; then
            CACHE_FLAGS="--cache-to=type=gha,mode=max --cache-from=type=gha"
          fi
          docker buildx build --platform ${plat} ${CACHE_FLAGS} --target base -t ${img_tag}-builder -f contrib/Dockerfile.debian-packaging --load "${REPO_ROOT}"

          echo "Running builder container to build binaries and create .deb for arch=${arch}"
          # Use per-arch target dir to prevent artifact clobbering
          docker run --rm --platform ${plat} -e CARGO_TARGET_DIR=/workdir/target-${arch} -v "${REPO_ROOT}":/workdir -w /workdir ${img_tag}-builder sh -c "cargo build --release && CARGO_TARGET_DIR=/workdir/target-${arch} cargo deb --no-strip --no-build"

        # Copy out artifacts
        mkdir -p "${OUTPUT_DIR}/debian/${arch}"
        DEBS=$(find "${REPO_ROOT}/target-${arch}/debian" -maxdepth 1 -type f -name "*.deb" -print || true)
        if [ -z "${DEBS}" ]; then
          echo "No .deb artifacts found for arch ${arch} in target-${arch}/debian/" >&2
        else
          for f in ${DEBS}; do
            cp -f "$f" "${OUTPUT_DIR}/debian/${arch}/"
          done
        fi

        # Create a minimal unix layout for other Linux distros: binary + systemd + config
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/usr/bin"
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system"
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn"

        # prefer per-arch target dir if present
        if [ -f "${REPO_ROOT}/target-${arch}/release/check_vpn" ]; then
          cp -f "${REPO_ROOT}/target-${arch}/release/check_vpn" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"
        elif [ -f "${REPO_ROOT}/target/release/check_vpn" ]; then
          cp -f "${REPO_ROOT}/target/release/check_vpn" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"
        else
          echo "Warning: no built binary found for arch ${arch} to copy into unix/" >&2
        fi

        # copy systemd unit and drop-ins if present
        if [ -f "${REPO_ROOT}/contrib/check_vpn.service" ]; then
          cp -f "${REPO_ROOT}/contrib/check_vpn.service" "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service"
        fi
        if [ -d "${REPO_ROOT}/contrib/systemd/check_vpn.service.d" ]; then
          mkdir -p "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d"
          cp -r "${REPO_ROOT}/contrib/systemd/check_vpn.service.d/"* "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d/" || true
        fi
        # copy default config/logrotate
        if [ -f "${REPO_ROOT}/contrib/config.xml" ]; then
          cp -f "${REPO_ROOT}/contrib/config.xml" "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn/config.xml"
        fi
        if [ -f "${REPO_ROOT}/contrib/logrotate.conf" ]; then
          mkdir -p "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d"
          cp -f "${REPO_ROOT}/contrib/logrotate.conf" "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d/check_vpn"
        fi
      done
      return 0
    else
      echo "Building Docker builder image: ${DOCKER_DEBIAN_TAG}-builder"
      docker build --target base -t ${DOCKER_DEBIAN_TAG}-builder -f contrib/Dockerfile.debian-packaging "${REPO_ROOT}"

      echo "Running builder container to build binaries and create .deb"
      docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}":/workdir -w /workdir ${DOCKER_DEBIAN_TAG}-builder sh -c "cargo build --release && CARGO_TARGET_DIR=/workdir/target cargo deb --no-strip --no-build"

      # create unix layout for non-buildx path
      mkdir -p "${OUTPUT_DIR}/unix/amd64/usr/bin"
      mkdir -p "${OUTPUT_DIR}/unix/amd64/lib/systemd/system"
      mkdir -p "${OUTPUT_DIR}/unix/amd64/etc/check_vpn"
      if [ -f "${REPO_ROOT}/target/release/check_vpn" ]; then
        cp -f "${REPO_ROOT}/target/release/check_vpn" "${OUTPUT_DIR}/unix/amd64/usr/bin/check_vpn"
      fi
      if [ -f "${REPO_ROOT}/contrib/check_vpn.service" ]; then
        cp -f "${REPO_ROOT}/contrib/check_vpn.service" "${OUTPUT_DIR}/unix/amd64/lib/systemd/system/check_vpn.service"
      fi
      if [ -d "${REPO_ROOT}/contrib/systemd/check_vpn.service.d" ]; then
        mkdir -p "${OUTPUT_DIR}/unix/amd64/lib/systemd/system/check_vpn.service.d"
        cp -r "${REPO_ROOT}/contrib/systemd/check_vpn.service.d/"* "${OUTPUT_DIR}/unix/amd64/lib/systemd/system/check_vpn.service.d/" || true
      fi
      if [ -f "${REPO_ROOT}/contrib/config.xml" ]; then
        cp -f "${REPO_ROOT}/contrib/config.xml" "${OUTPUT_DIR}/unix/amd64/etc/check_vpn/config.xml"
      fi
    fi
  else
    if ! command -v cargo-deb >/dev/null 2>&1; then
      echo "cargo-deb not found locally; install it or run with Docker" >&2
      exit 1
    fi

    (cd "${REPO_ROOT}" && ./contrib/cargo-deb.sh)
  fi

  # Copy debs (non-buildx path)
  DEBS=$(find "${REPO_ROOT}/target/debian" -maxdepth 1 -type f -name "*.deb" -print || true)
  if [ -z "${DEBS}" ]; then
    echo "No .deb artifacts found in target/debian/" >&2
  else
    for f in ${DEBS}; do
      cp -f "$f" "${OUTPUT_DIR}/debian/"
    done
  fi
}

run_fedora() {
  echo "=== Fedora packaging (docker=${USE_DOCKER}) ==="
  if [ "$USE_DOCKER" -eq 1 ]; then
    if command -v docker buildx >/dev/null 2>&1 || docker buildx version >/dev/null 2>&1; then
      if ! docker buildx inspect > /dev/null 2>&1; then
        docker buildx create --use --name buildx-builder || true
      fi

      for plat in "${PLATFORM_ARRAY[@]}"; do
        arch=$(echo "$plat" | awk -F'/' '{print $2}')
        img_tag="${DOCKER_FEDORA_TAG}-${arch}"
        echo "Building image for platform ${plat} -> ${img_tag}"
        CACHE_FLAGS=""
        if [ "${BUILD_CACHE}" = "1" ]; then
          CACHE_FLAGS="--cache-to=type=gha,mode=max --cache-from=type=gha"
        fi
        docker buildx build --platform ${plat} ${CACHE_FLAGS} --target base -t ${img_tag}-builder -f contrib/Dockerfile.fedora-packaging --load "${REPO_ROOT}"

        echo "Running builder container to build binaries and create .rpm for arch=${arch}"
        docker run --rm --platform ${plat} -e CARGO_TARGET_DIR=/workdir/target-${arch} -v "${REPO_ROOT}":/workdir -w /workdir ${img_tag}-builder sh -c "CARGO_TARGET_DIR=/workdir/target-${arch} cargo build --release && CARGO_TARGET_DIR=/workdir/target-${arch} ./contrib/fpm-build.sh \"${VERSION}\""

        mkdir -p "${OUTPUT_DIR}/fedora/${arch}"
        RPMS=$(find "${REPO_ROOT}" -maxdepth 1 -type f -name "*.rpm" -print || true)
        if [ -z "${RPMS}" ]; then
          echo "No .rpm artifacts found for arch ${arch}" >&2
        else
          for f in ${RPMS}; do
            cp -f "$f" "${OUTPUT_DIR}/fedora/${arch}/"
            # remove rpms so next arch run starts clean
            rm -f "$f" || true
          done
        # also produce unix layout from the per-arch build if needed
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/usr/bin"
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system"
        mkdir -p "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn"
        if [ -f "${REPO_ROOT}/target-${arch}/release/check_vpn" ]; then
          cp -f "${REPO_ROOT}/target-${arch}/release/check_vpn" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"
        elif [ -f "${REPO_ROOT}/target/release/check_vpn" ]; then
          cp -f "${REPO_ROOT}/target/release/check_vpn" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"
        fi
        if [ -f "${REPO_ROOT}/contrib/check_vpn.service" ]; then
          cp -f "${REPO_ROOT}/contrib/check_vpn.service" "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service"
        fi
        if [ -d "${REPO_ROOT}/contrib/systemd/check_vpn.service.d" ]; then
          mkdir -p "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d"
          cp -r "${REPO_ROOT}/contrib/systemd/check_vpn.service.d/"* "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d/" || true
        fi
        if [ -f "${REPO_ROOT}/contrib/config.xml" ]; then
          cp -f "${REPO_ROOT}/contrib/config.xml" "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn/config.xml"
        fi
        if [ -f "${REPO_ROOT}/contrib/logrotate.conf" ]; then
          mkdir -p "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d"
          cp -f "${REPO_ROOT}/contrib/logrotate.conf" "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d/check_vpn"
        fi
        fi
      done
      return 0
    else
      echo "Building Docker builder image: ${DOCKER_FEDORA_TAG}-builder"
      docker build --target base -t ${DOCKER_FEDORA_TAG}-builder -f contrib/Dockerfile.fedora-packaging "${REPO_ROOT}"

      echo "Running builder container to build binaries and create .rpm"
      docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}":/workdir -w /workdir ${DOCKER_FEDORA_TAG}-builder sh -c "CARGO_TARGET_DIR=/workdir/target cargo build --release && CARGO_TARGET_DIR=/workdir/target ./contrib/fpm-build.sh \"${VERSION}\""
    fi
  else
    if ! command -v fpm >/dev/null 2>&1; then
      echo "fpm not found locally; install it (gem install fpm) or run with Docker" >&2
      exit 1
    fi

    (cd "${REPO_ROOT}" && ./contrib/fpm-build.sh "${VERSION}")
  fi

  # Copy rpms (the contrib/fpm-build.sh prints them to the current dir)
  RPMS=$(find "${REPO_ROOT}" -maxdepth 1 -type f -name "*.rpm" -print || true)
  if [ -z "${RPMS}" ]; then
    echo "No .rpm artifacts found in repository root" >&2
  else
    for f in ${RPMS}; do
      cp -f "$f" "${OUTPUT_DIR}/fedora/"
    done
  fi
}

if [ "$SKIP_DEBIAN" -eq 0 ]; then
  run_debian
else
  echo "Skipping Debian packaging"
fi

if [ "$SKIP_FEDORA" -eq 0 ]; then
  run_fedora
else
  echo "Skipping Fedora packaging"
fi

echo "Done — artifacts are under: ${OUTPUT_DIR}"
