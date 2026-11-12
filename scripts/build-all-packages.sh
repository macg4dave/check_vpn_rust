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
    --version|-v|--output-dir|--platforms)
      if [ "$#" -lt 2 ]; then echo "Missing value for $1" >&2; exit 2; fi ;;
  esac
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

VERSION="$(bash "${REPO_ROOT}/scripts/package-version.sh" "$VERSION")"

VERSION_NORMALIZED="${VERSION#v}"
DEB_VERSION_FULL="${VERSION_NORMALIZED}-${DEB_REVISION}"
VERSION_FILE_SAFE="$VERSION"

[ -n "$OUTPUT_DIR" ] || { echo "Output directory must not be empty" >&2; exit 2; }
[[ "$DEB_REVISION" =~ ^[1-9][0-9]*$ ]] || { echo "Debian revision must be a positive integer" >&2; exit 2; }
IFS=',' read -r -a PLATFORM_ARRAY <<< "$PLATFORMS"
[ "${#PLATFORM_ARRAY[@]}" -gt 0 ] || { echo "At least one platform is required" >&2; exit 2; }
for platform in "${PLATFORM_ARRAY[@]}"; do
  case "$platform" in linux/amd64|linux/arm64) ;; *) echo "Unsupported platform: $platform" >&2; exit 2 ;; esac
done
mkdir -p "$OUTPUT_DIR/debian" "$OUTPUT_DIR/fedora" "$OUTPUT_DIR/unix" "$OUTPUT_DIR/bin"

die() {
  echo "$*" >&2
  exit 1
}

have_buildx() {
  docker buildx version >/dev/null 2>&1
}

ensure_buildx_builder() {
  docker buildx inspect >/dev/null 2>&1 || docker buildx create --use --name check-vpn-builder >/dev/null
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
  else
    die "Missing release binary: ${target_dir}/release/check_vpn"
  fi
}

copy_unix_layout() {
  local arch="$1"
  local target_dir="$2"

  mkdir -p "${OUTPUT_DIR}/unix/${arch}/usr/bin" \
    "${OUTPUT_DIR}/unix/${arch}/lib/systemd/system" \
    "${OUTPUT_DIR}/unix/${arch}/etc/check_vpn" \
    "${OUTPUT_DIR}/unix/${arch}/usr/share/doc/check_vpn"
  cp "${REPO_ROOT}/INSTALL.md" "${OUTPUT_DIR}/unix/${arch}/usr/share/doc/check_vpn/"

  local bin_source
  bin_source=$(find_built_binary "$target_dir")
  cp -f "$bin_source" "${OUTPUT_DIR}/unix/${arch}/usr/bin/check_vpn"

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

  local dest_dir="${OUTPUT_DIR}/bin/${arch}"
  local tar_name="check_vpn-${VERSION_FILE_SAFE}-${arch}.tar.gz"

  mkdir -p "$dest_dir"
  cp -f "$bin_source" "${dest_dir}/check_vpn"
  tar -czf "${dest_dir}/${tar_name}" -C "$(dirname "$bin_source")" "$(basename "$bin_source")"
}

copy_debs_from() {
  local src_dir="$1"
  local dest_dir="$2"
  local arch="${3:-$(detect_host_arch)}"
  local files=("$src_dir"/*_"$DEB_VERSION_FULL"_"$arch".deb)
  [ -f "${files[0]}" ] || die "No .deb artifacts found in ${src_dir}"
  mkdir -p "$dest_dir"
  cp -- "${files[@]}" "$dest_dir/"
}

copy_rpms_from_root() {
  local dest_dir="$1"
  local remove_after="${2:-false}"
  local arch="${3:-$(detect_host_arch)}"
  case "$arch" in amd64) arch=x86_64 ;; arm64) arch=aarch64 ;; esac
  local files=("${REPO_ROOT}"/*-"$VERSION_NORMALIZED"-*."$arch".rpm)
  [ -f "${files[0]}" ] || die "No .rpm artifacts found in repository root"
  mkdir -p "$dest_dir"
  cp -- "${files[@]}" "$dest_dir/"
  if [ "$remove_after" = "true" ]; then
    rm -- "${files[@]}"
  fi
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
          build_args+=("--cache-to=type=gha,mode=max" "--cache-from=type=gha")
        fi
        build_args+=("${REPO_ROOT}")
        "${build_args[@]}"

        echo "Running builder container to build binaries and create .deb for arch=${arch}"
        docker run --rm --platform "$plat" -e CARGO_TARGET_DIR=/workdir/target-"${arch}" -v "${REPO_ROOT}:/workdir" -w /workdir "${img_tag}-builder" \
          sh -c "cargo build --release --locked && CARGO_TARGET_DIR=/workdir/target-${arch} cargo deb --no-strip --no-build --deb-version \"${DEB_VERSION_FULL}\""

        copy_debs_from "${REPO_ROOT}/target-${arch}/debian" "${OUTPUT_DIR}/debian/${arch}" "$arch"
        copy_unix_layout "${arch}" "${REPO_ROOT}/target-${arch}"
        copy_binary_artifacts "${arch}" "${REPO_ROOT}/target-${arch}"
      done
      return
    fi

    echo "Building Docker builder image: ${DOCKER_DEBIAN_TAG}-builder"
    docker build --target base -t "${DOCKER_DEBIAN_TAG}-builder" -f contrib/Dockerfile.debian-packaging "${REPO_ROOT}"

    echo "Running builder container to build binaries and create .deb"
    docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}:/workdir" -w /workdir "${DOCKER_DEBIAN_TAG}-builder" \
      sh -c "cargo build --release --locked && CARGO_TARGET_DIR=/workdir/target cargo deb --no-strip --no-build --deb-version \"${DEB_VERSION_FULL}\""
  else
    command -v cargo-deb >/dev/null 2>&1 || die "cargo-deb not found locally; install it or run with Docker"
    (cd "${REPO_ROOT}" && cargo build --release --locked && cargo deb --no-strip --deb-version "${DEB_VERSION_FULL}")
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
          build_args+=("--cache-to=type=gha,mode=max" "--cache-from=type=gha")
        fi
        build_args+=("${REPO_ROOT}")
        "${build_args[@]}"

        echo "Running builder container to build binaries and create .rpm for arch=${arch}"
        docker run --rm --platform "$plat" -e CARGO_TARGET_DIR=/workdir/target-"${arch}" -v "${REPO_ROOT}:/workdir" -w /workdir "${img_tag}-builder" \
          sh -c "CARGO_TARGET_DIR=/workdir/target-${arch} cargo build --release --locked && CARGO_TARGET_DIR=/workdir/target-${arch} ./contrib/fpm-build.sh \"${VERSION_NORMALIZED}\""

        copy_rpms_from_root "${OUTPUT_DIR}/fedora/${arch}" true "$arch"
        copy_unix_layout "${arch}" "${REPO_ROOT}/target-${arch}"
        copy_binary_artifacts "${arch}" "${REPO_ROOT}/target-${arch}"
      done
      return
    fi

    echo "Building Docker builder image: ${DOCKER_FEDORA_TAG}-builder"
    docker build --target base -t "${DOCKER_FEDORA_TAG}-builder" -f contrib/Dockerfile.fedora-packaging "${REPO_ROOT}"

    echo "Running builder container to build binaries and create .rpm"
    docker run --rm -e CARGO_TARGET_DIR=/workdir/target -v "${REPO_ROOT}:/workdir" -w /workdir "${DOCKER_FEDORA_TAG}-builder" \
      sh -c "CARGO_TARGET_DIR=/workdir/target cargo build --release --locked && CARGO_TARGET_DIR=/workdir/target ./contrib/fpm-build.sh \"${VERSION_NORMALIZED}\""
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


echo "Done - artifacts are under: ${OUTPUT_DIR}"
