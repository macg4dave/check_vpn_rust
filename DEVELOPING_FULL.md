## Developing and building check_vpn (platform requirements & guide)

This document collects specific build/runtime requirements and step-by-step instructions for all supported platforms: macOS, Debian (via Docker), Fedora (via Docker), and Windows (native + WSL2).

If you need a short summary, use `DEVELOPING.md` top-level commands. This file expands the requirements and commands so contributors can reproduce builds and tests reliably.

### Table of contents
- System requirements (per OS)
- Toolchain installation (Rust + toolchains)
- Native dependencies (OpenSSL, libcurl, etc.)
- Building locally (per OS)
- Docker-based reproducible builds (Debian/Fedora)
- CI notes (GitHub Actions)
- Troubleshooting and common errors

## System requirements (per OS)

### macOS (developer machine)
- macOS 12+ recommended
- Homebrew package manager (https://brew.sh)
- Xcode Command Line Tools: `xcode-select --install`
- rustup (see Toolchain installation)

### Debian / Ubuntu (for local builds or inside Docker)
- Docker Desktop (on Windows/macOS) or native Docker Engine on Linux
- If building natively (not recommended for packaging parity), install system packages listed in the Dockerfile (see section below).

### Fedora (for packaging parity)
- Docker Desktop or native Docker Engine on Linux

### Windows (native)
- Windows 10/11 (latest updates)
- rustup (https://rustup.rs)
- Visual Studio Build Tools (C++ workload) for MSVC toolchain, or MSYS2 for GNU toolchain
- If using WSL2: install Ubuntu or another distro and enable Docker Desktop WSL integration

### Common: Docker (for reproducible Linux builds and tests)
- Docker Desktop for Windows/macOS: install and start the Linux engine. Enable WSL 2 integration if you run scripts from WSL.
- On Linux: install docker.io/dockerd and ensure the daemon is running (systemd service).

## Toolchain installation

Install Rust via rustup (works across platforms):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# then in a new shell
rustup default stable
```

### Windows specifics
- Recommended: use MSVC toolchain for best compatibility with native libraries:

```powershell
rustup default stable-x86_64-pc-windows-msvc
```

- Install Visual Studio Build Tools (C++ workload). Download from Microsoft and ensure `cl.exe` is available on PATH.

### WSL2 specifics
- If using WSL2, install rustup inside the WSL distro (not on Windows) for Linux-native builds, or use the Dockerfiles for exact parity.

## Native dependencies (system libraries)

This project uses crates that may require system libraries (libssl, libcurl, etc.). The Dockerfiles in `contrib/` install the necessary packages for Debian/Fedora. If building natively, install the following as a starting point.

### Debian/Ubuntu (native):

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev libcurl4-openssl-dev libgmp-dev
```

### Fedora (native):

```bash
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkgconfig openssl-devel libcurl-devel gmp-devel
```

### macOS (Homebrew):

```bash
brew install openssl pkg-config curl
# Set environment to let cargo find OpenSSL (if needed):
export OPENSSL_DIR="$(brew --prefix openssl)"
```

### Windows (MSVC)
- Ensure Visual Studio Build Tools installed. For OpenSSL, you can use vcpkg or a prebuilt binary. Example vcpkg flow (optional):

```powershell
# clone vcpkg and bootstrap
# bootstrap vcpkg and install OpenSSL
# use the vcpkg toolchain file when building if needed
```

Note: the exact system packages depend on enabled Cargo features (curl, openssl, etc.). Inspect `Cargo.toml` and `contrib/Dockerfile.*` for the set of packages used by CI images.

## Building locally (per OS)

General commands (repo root):

```bash
# Run the app (development)
cargo run

# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features
```

### macOS (native)
- Ensure Xcode Command Line Tools and Homebrew deps installed. Then run `cargo build --release`.

### Windows (native)
- From PowerShell (MSVC toolchain):

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-windows.ps1 -Release
```

### Windows (WSL2)
- Use WSL shell and either `make` or the shell test scripts. WSL will use Linux toolchain and Docker integration if enabled.

## Docker-based reproducible builds (Debian / Fedora)

We provide Dockerfiles in `contrib/` that install the system deps and run the build inside a clean environment. This is the recommended way to produce Linux artifacts for release.

Convenience targets & scripts:
- `Makefile` targets: `debian-build-release`, `fedora-build-release`
- PowerShell: `scripts/build-debian-docker.ps1`, `scripts/build-fedora-docker.ps1`
- Test runners: `scripts/test-all-docker.ps1` (PowerShell) and `scripts/test-all-docker.sh` (POSIX)

Typical Docker commands (manual):

```bash
docker build -t check_vpn_debian -f contrib/Dockerfile.debian .
docker run --rm -v "$PWD":/work -w /work check_vpn_debian bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"
```

If Docker is installed but the daemon is not running, start Docker Desktop (Windows/macOS) or `sudo systemctl start docker` on Linux.

## CI notes (GitHub Actions)

We use `ci-matrix.yml` which builds on `ubuntu-latest`, `macos-latest`, and `windows-latest`:
- On `ubuntu-latest` the workflow uses Docker to produce Debian/Fedora artifacts.
- On `macos-latest` the workflow runs a native `cargo build --release`.
- On `windows-latest` the workflow runs `scripts/build-windows.ps1` to produce a windows zip.

The workflow uploads the built artifacts as release-ready tarballs or zip files.

## Troubleshooting and common errors

- Docker CLI present but daemon not responding: start Docker Desktop (Windows/macOS) or systemd docker service on Linux.
- Missing native libraries (OpenSSL/libcurl): install the platform-specific dev packages listed above or enable Cargo features that vendor dependencies.
- On Windows, if `cl.exe` not found, install Visual Studio Build Tools C++ workload and ensure the developer command prompt environment is used or Visual Studio tools are on PATH.
- If a test interacts with the network and is marked `#[ignore]`, run ignored tests explicitly:

```bash
cargo test -- --ignored
```

## Make and script shortcuts

- Use `make` on Unix or WSL to run targets defined in `Makefile` (requires GNU make).
- Windows users without `make` can use `scripts/build-all.ps1` to build all artifacts or `scripts/test-all-docker.ps1` to run tests in Docker images.

## Files added by this repository to support these flows

- `Makefile` — convenience tasks for local dev and Docker builds (targets: `all`, `release`, `debian-build-release`, `fedora-build-release`, `test-all-docker`, `windows-test-all-docker`).
- `scripts/*.ps1` — PowerShell helpers for Windows builds and Docker operations.
- `scripts/*.sh` — POSIX shell counterparts for WSL/Linux/macOS.
- `.github/workflows/ci-matrix.yml` — CI matrix workflow for cross-platform builds.

---

If you want I can further tighten this document: produce a checklist for each OS, or add one-line install snippets for popular package managers (apt, dnf, brew, choco). Which would you prefer?
