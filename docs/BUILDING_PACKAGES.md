# Building distribution packages (Debian / Fedora)

This repository includes reproducible packaging helpers that build Debian (.deb)
and Fedora (.rpm) packages inside Docker images so you don't have to install
system packaging dependencies locally.

Top-level script
-----------------

Use the orchestrator script to build both package types and place artifacts in
the `release/` folder:

```sh
# Build both packages (default: tries to derive version from git tags)
./scripts/build-all-packages.sh

# Build both packages for a specific version
./scripts/build-all-packages.sh --version 1.2.3

# Run packaging without Docker (requires `cargo-deb` and `fpm` installed)
./scripts/build-all-packages.sh --no-docker

# Build only Debian or only Fedora packages
./scripts/build-all-packages.sh --skip-fedora
./scripts/build-all-packages.sh --skip-debian
```

Docker images
-------------

Two images are provided to keep packaging reproducible:

- `contrib/Dockerfile.debian-packaging` - Debian-based image with rustup and `cargo-deb` installed to build .deb packages.
- `contrib/Dockerfile.fedora-packaging` - Fedora-based image with rustup and `fpm` (Ruby gem) to build .rpm packages.

The packaging script builds these images and executes the existing packaging helper
scripts in `contrib/` (`contrib/cargo-deb.sh` and `contrib/fpm-build.sh`).

Artifacts
---------

After a successful run, you will find packages in:

- `release/debian/` - contains `*.deb` files created by `cargo-deb`.
- `release/fedora/` - contains `*.rpm` files created by `fpm`.

Make targets
------------

There are convenience Makefile targets that call the orchestrator script:

- `make package-all`
- `make package-debian`
- `make package-fedora`

Cross-architecture (multi-arch) builds
-------------------------------------

The packaging script supports multi-arch builds via Docker BuildKit / buildx. By default
it will target both linux/amd64 and linux/arm64 when buildx is available. Artifacts are
placed into per-arch subfolders under `release/`, for example:

- `release/debian/amd64/` and `release/debian/arm64/`
- `release/fedora/amd64/` and `release/fedora/arm64/`

Example (explicit platforms):

```sh
./scripts/build-all-packages.sh --platforms linux/amd64,linux/arm64 --version 1.2.3
```

Notes: buildx needs to be available and configured on the machine (CI runners usually
have it); on local machines you may need to enable `docker buildx` and set up QEMU
emulation for cross-arch image execution. If buildx isn't available the script falls
back to a regular local Docker build for the host platform.

CI & build cache
-----------------

The project includes a CI job `CI - Package (multi-arch)` which runs `make package-all`
and produces per-arch artifacts; this job uses BuildKit's buildx with QEMU and enables
cache storage for faster repeated builds. On CI we enable the build cache via the
environment variable `BUILD_CACHE=1` so the packaging script passes `--cache-to=type=gha`
and `--cache-from=type=gha` to buildx. This makes subsequent builds faster by reusing
layers stored in GitHub Actions buildx cache.

If you run the script locally and want to enable buildx caching, ensure your `docker
buildx` has a builder driver and that `docker buildx` supports `cache-to=type=gha`.

Image size optimizations
-------------------------

Packaging Docker images are optimized by pruning cargo/rustup caches and stripping
binaries where possible - this reduces image size without removing required runtime
tooling. The images deliberately keep the rust toolchain and packaging tools that are
required to compile and build packages inside the container at runtime, but we remove
download caches and temporary artifacts to keep images as small as practical.

Notes & requirements
---------------------
- Docker must be available and runnable for the default behaviour. When Docker
  isn't available, pass `--no-docker` to run packaging locally (you must have
  `cargo-deb` for Debian and `fpm` for RPM installed).
- For reproducible builds in CI, prefer the Docker-based packaging path.