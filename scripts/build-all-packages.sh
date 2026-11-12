#!/usr/bin/env bash
set -euo pipefail

# This script uses bash features (arrays). If invoked via `sh`, fail fast with a clear error.
if [ -z "${BASH_VERSION:-}" ]; then
  echo "Error: this script must be run with bash (e.g. 'bash ./scripts/build-all-packages.sh')." >&2
  exit 2
fi

# Reproducible packaging helper. Builds Debian (.deb) and Fedora (.rpm) packages
# inside Docker images and places artifacts into release/debian/ and
# release/fedora/ respectively.
#
# Usage:
#  ./scripts/build-all-packages.sh [--version X.Y.Z] [--output-dir ./release] [--no-docker] [--skip-debian] [--skip-fedora]

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DOCKER_DEBIAN_TAG="check_vpn_debian_packaging"
DOCKER_FEDORA_TAG="check_vpn_fedora_packaging"
# Buildx cache behavior if BUILD_CACHE=1
BUILD_CACHE="${BUILD_CACHE:-0}"
OUTPUT_DIR_DEFAULT="${REPO_ROOT}/release"
PLATFORMS="${PLATFORMS:-linux/amd64,linux/arm64}"
VERSION=""
DEB_REVISION="${DEB_REVISION:-1}"
OUTPUT_DIR="$OUTPUT_DIR_DEFAULT"
USE_DOCKER=1
SKIP_DEBIAN=0
SKIP_FEDORA=0

usage() {
  cat <<EOF_USAGE
Usage: $0 [--version X.Y.Z] [--output-dir DIR] [--no-docker] [--skip-debian] [--skip-fedora] [--platforms list]

By default this script builds both Debian (.deb) and Fedora (.rpm) packages in Docker
containers and writes them to ${OUTPUT_DIR_DEFAULT}/debian and ${OUTPUT_DIR_DEFAULT}/fedora.
Use --no-docker to attempt local packaging (requires cargo-deb and fpm installed locally).
EOF_USAGE
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
      usage; exit 0;;
    *)
      echo "Unknown argument: $1" >&2; usage; exit 2;;
  esac
done

if [ -z "$VERSION" ]; then
  if command -v git >/dev/null 2>&1 && [ -d "${REPO_ROOT}/.git" ]; then
    VERSION=$(git -C "${REPO_ROOT}" describe --tags --abbrev=0 2>/dev/null || true)
  fi
  VERSION=${VERSION:-0.0.0}
fi

VERSION_NORMALIZED="${VERSION#v}"
DEB_VERSION_FULL="${VERSION_NORMALIZED}-${DEB_REVISION}"
VERSION_FILE_SAFE="${VERSION//\//_}"
VERSION_FILE_SAFE="${VERSION_FILE_SAFE//:/_}"

mkdir -p "$OUTPUT_DIR/debian" "$OUTPUT_DIR/fedora" "$OUTPUT_DIR/unix" "$OUTPUT_DIR/bin"

IFS=',' read -r -a PLATFORM_ARRAY <<< "$PLATFORMS"

die() {
  echo "$*" >&2
  exit 1
}

have_buildx() {
  docker buildx version >/dev/null 2>&1
}

ensure_buildx_builder() {
  docker buildx inspect >/dev/null 2>&1 || docker buildx create --use --name check-vpn-builder >/dev/null 2>&1 || true
}

detect_host_arch() {
  local machine
  machine="$(uname -m 2>/dev/null || echo unknown)"
  case "$machine" in
    x86_64|amd64) echo "amd64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) echo "$machine" ;;
  esac
}

find_built_binary() {
  local target_dir="$1"

  if [ -f "${target_dir}/release/check_vpn" ]; then
    echo "${target_dir}/release/check_vpn"
  elif [ -f "${REPO_ROOT}/target/release/check_vpn" ]; then
    echo "${REPO_ROOT}/target/release/check_vpn"
  else
    echo ""
  fi
}

copy_unix_layout() {
  local arch="$1"
  local target_dir="$2"

  mkdir -p "${OUTPUT_DIR}/unix/${arch}/usr/bin" \
    "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system" \
    "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn" \
    "${OUTPUT_DIR}/unix/${arch}/usr/share/doc/check_vpn"
  cp "${REPO_ROOT}/docs/RECOVERY_PERMISSIONS.md" "${OUTPUT_DIR}/unix/${arch}/usr/share/doc/check_vpn/"

  local bin_source
  bin_source=$(find_built_binary "$target_dir")

  if [ -n "$bin_source" ]; then
    cp -f "$bin_source" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"
  else
    echo "Warning: no built binary found for arch ${arch} to copy into unix/" >&2
  fi

  if [ -f "${REPO_ROOT}/contrib/check_vpn.service" ]; then
    cp -f "${REPO_ROOT}/contrib/check_vpn.service" "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service"
  fi
  if [ -d "${REPO_ROOT}/contrib/systemd/check_vpn.service.d" ]; then
    mkdir -p "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d"
    cp -r "${REPO_ROOT}/contrib/systemd/check_vpn.service.d/"* "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system/check_vpn.service.d/" 2>/dev/null || true
  fi
  if [ -f "${REPO_ROOT}/contrib/config.xml" ]; then
    cp -f "${REPO_ROOT}/contrib/config.xml" "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn/config.xml"
  fi
  if [ -f "${REPO_ROOT}/contrib/logrotate.conf" ]; then
    mkdir -p "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d"
    cp -f "${REPO_ROOT}/contrib/logrotate.conf" "${OUTPUT_DIR}/unix/${arch}/etc/logrotate.d/check_vpn"
  fi
}

copy_binary_artifacts() {
  local arch="$1"
  local target_dir="$2"
  local bin_source

  bin_source=$(find_built_binary "$target_dir")
  if [ -z "$bin_source" ]; then
    echo "Warning: no built binary found for arch ${arch} to copy into release/bin" >&2
    return
  fi

  local dest_dir="${OUTPUT_DIR}/bin/${arch}"
  local tar_name="check_vpn-${VERSION_FILE_SAFE}-${arch}.tar.gz"

  mkdir -p "$dest_dir"
  cp -f "$bin_source" "${dest_dir}/check_vpn"
  tar -czf "${dest_dir}/${tar_name}" -C "$(dirname "$bin_source")" "$(basename "$bin_source")"
}

copy_debs_from() {
  local src_dir="$1"
  local dest_dir="$2"
  local debs

  debs=$(find "$src_dir" -maxdepth 1 -type f -name "*.deb" -print 2>/dev/null || true)
  if [ -z "$debs" ]; then
    echo "No .deb artifacts found in ${src_dir}" >&2
    return
  fi

  mkdir -p "$dest_dir"
  for f in $debs; do
    cp -f "$f" "$dest_dir/"
  done
}

copy_rpms_from_root() {
  local dest_dir="$1"
  local remove_after="${2:-false}"
  local rpms

  rpms=$(find "${REPO_ROOT}" -maxdepth 1 -type f -name "*.rpm" -print 2>/dev/null || true)
  if [ -z "$rpms" ]; then
    echo "No .rpm artifacts found in repository root" >&2
    return
  fi

  mkdir -p "$dest_dir"
  for f in $rpms; do
    cp -f "$f" "$dest_dir/"
    if [ "$remove_after" = "true" ]; then
      rm -f "$f" || true
    fi
  done
}

normalize_release_filenames() {
  # Debian: replace trailing _amd64.deb in arm64 dir to _arm64.deb
  find "${OUTPUT_DIR}/debian" -type f -name '*_amd64.deb' 2>/dev/null | while read -r f; do
    if [[ "$f" == *"/arm64/"* ]]; then
      local new
      new="${f%_amd64.deb}_arm64.deb"
      mv -f "$f" "$new" || true
    fi
  done || true

  # Fedora: for arm64 replace x86_64 -> aarch64
  find "${OUTPUT_DIR}/fedora" -type f -name '*x86_64.rpm' 2>/dev/null | while read -r f; do
    if [[ "$f" == *"/arm64/"* ]]; then
      local new
      new="${f%x86_64.rpm}aarch64.rpm"
      mv -f "$f" "$new" || true
    fi
  done || true
}

run_debian() {
  echo "=== Debian packaging (docker=${USE_DOCKER}) ==="
  if [ "$USE_DOCKER" -eq 1 ]; then
    if have_buildx; then
      ensure_buildx_builder
      for plat in "${PLATFORM_ARRAY[@]}"; do
        local arch
        arch="${plat#*/}"
        local img_tag
        img_tag="${DOCKER_DEBIAN_TAG}-${arch}"

        echo "Building image for platform ${plat} -> ${img_tag}"
        build_args=(docker buildx build --load --platform "$plat" --target base -t "${img_tag}-builder" -f contrib/Dockerfile.debian-packaging)
        if [ "$BUILD_CACHE" = "1" ]; then
          build_args+=(--cache-to=type=gha,mode=max --cache-from=type=gha)
        fi
        build_args+=("${REPO_ROOT}")
        "${build_args[@]}"

        echo "Running builder container to build binaries and create .deb for arch=${arch}"
        docker run --rm --platform "$plat" -e CARGO_TARGET_DIR=/workdir/target-"${arch}" -v "${REPO_ROOT}:/workdir" -w /workdir "${img_tag}-builder" \
          sh -c "cargo build --release && CARGO_TARGET_DIR=/workdir/target-${arch} cargo deb --no-strip --no-build --deb-version \"${DEB_VERSION_FULL}\""

        copy_debs_from "${REPO_ROOT}/target-${arch}/debian" "${OUTPUT_DIR}/debian/${arch}"
        copy_unix_layout "${arch}" "${REPO_ROOT}/target-${arch}"
        copy_binary_artifacts "${arch}" "${REPO_ROOT}/target-${arch}"
      done
      return
    fi

    echo "Building Docker builder image: ${DOCKER_DEBIAN_TAG}-builder"
    docker build --target base -t "${DOCKER_DEBIAN_TAG}-builder" -f contrib/Dockerfile.debian-packaging "${REPO_ROOT}"

    echo "Running builder container to build binaries and create .deb"
    docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}:/workdir" -w /workdir "${DOCKER_DEBIAN_TAG}-builder" \
      sh -c "cargo build --release && CARGO_TARGET_DIR=/workdir/target cargo deb --no-strip --no-build --deb-version \"${DEB_VERSION_FULL}\""
  else
    command -v cargo-deb >/dev/null 2>&1 || die "cargo-deb not found locally; install it or run with Docker"
    (cd "${REPO_ROOT}" && cargo build --release && cargo deb --no-strip --deb-version "${DEB_VERSION_FULL}")
  fi

  local host_arch
  host_arch="$(detect_host_arch)"

  copy_debs_from "${REPO_ROOT}/target/debian" "${OUTPUT_DIR}/debian"
  copy_unix_layout "${host_arch}" "${REPO_ROOT}/target"
  copy_binary_artifacts "${host_arch}" "${REPO_ROOT}/target"
}

run_fedora() {
  echo "=== Fedora packaging (docker=${USE_DOCKER}) ==="
  if [ "$USE_DOCKER" -eq 1 ]; then
    if have_buildx; then
      ensure_buildx_builder
      for plat in "${PLATFORM_ARRAY[@]}"; do
        local arch
        arch="${plat#*/}"
        local img_tag
        img_tag="${DOCKER_FEDORA_TAG}-${arch}"

        echo "Building image for platform ${plat} -> ${img_tag}"
        build_args=(docker buildx build --load --platform "$plat" --target base -t "${img_tag}-builder" -f contrib/Dockerfile.fedora-packaging)
        if [ "$BUILD_CACHE" = "1" ]; then
          build_args+=(--cache-to=type=gha,mode=max --cache-from=type=gha)
        fi
        build_args+=("${REPO_ROOT}")
        "${build_args[@]}"

        echo "Running builder container to build binaries and create .rpm for arch=${arch}"
        docker run --rm --platform "$plat" -e CARGO_TARGET_DIR=/workdir/target-"${arch}" -v "${REPO_ROOT}:/workdir" -w /workdir "${img_tag}-builder" \
          sh -c "CARGO_TARGET_DIR=/workdir/target-${arch} cargo build --release && CARGO_TARGET_DIR=/workdir/target-${arch} ./contrib/fpm-build.sh \"${VERSION_NORMALIZED}\""

        copy_rpms_from_root "${OUTPUT_DIR}/fedora/${arch}" true
        copy_unix_layout "${arch}" "${REPO_ROOT}/target-${arch}"
        copy_binary_artifacts "${arch}" "${REPO_ROOT}/target-${arch}"
      done
      return
    fi

    echo "Building Docker builder image: ${DOCKER_FEDORA_TAG}-builder"
    docker build --target base -t "${DOCKER_FEDORA_TAG}-builder" -f contrib/Dockerfile.fedora-packaging "${REPO_ROOT}"

    echo "Running builder container to build binaries and create .rpm"
    docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}:/workdir" -w /workdir "${DOCKER_FEDORA_TAG}-builder" \
      sh -c "CARGO_TARGET_DIR=/workdir/target cargo build --release && CARGO_TARGET_DIR=/workdir/target ./contrib/fpm-build.sh \"${VERSION_NORMALIZED}\""
  else
    command -v fpm >/dev/null 2>&1 || die "fpm not found locally; install it (gem install fpm) or run with Docker"
    (cd "${REPO_ROOT}" && ./contrib/fpm-build.sh "${VERSION_NORMALIZED}")
  fi

  local host_arch
  host_arch="$(detect_host_arch)"

  copy_rpms_from_root "${OUTPUT_DIR}/fedora"
  copy_unix_layout "${host_arch}" "${REPO_ROOT}/target"
  copy_binary_artifacts "${host_arch}" "${REPO_ROOT}/target"
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

normalize_release_filenames

echo "Done - artifacts are under: ${OUTPUT_DIR}"
