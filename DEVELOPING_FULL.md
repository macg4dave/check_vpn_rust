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

### Windows (native notes)
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

```
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

```
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
Note: we have removed PowerShell-specific helper scripts from the repository. To produce Windows artifacts you can either use the CI on `windows-latest` or build on a native Windows machine using the Visual Studio toolchain. If you prefer a POSIX workflow, use WSL2 and run the normal POSIX helpers from a WSL shell (see "Windows (WSL2)" below).

### Windows (WSL2)
- Use WSL shell and either `make` or the POSIX shell test/build scripts under `./scripts`. WSL will use the Linux toolchain and (if enabled) Docker Desktop's WSL integration. From Windows you can invoke WSL commands directly (e.g. `wsl make` or `wsl ./scripts/test-all-docker.sh`).

See also `make help` for a summary, or use the sub-commands `make help-dev`, `make help-docker`, `make help-packaging`, and `make help-windows` for focused help, or `make help-targets` for extended Makefile help (includes the WSL guard for `dev-all`).

Note: The `make dev-all` shortcut includes a guard that prevents running this target from a native Windows shell (PowerShell/CMD/Git Bash) by default; this reduces accidental environment mismatches. To run it from a native Windows shell (not recommended) set `ALLOW_NON_WSL=1`:

```bash
ALLOW_NON_WSL=1 make dev-all
```

From PowerShell/CMD:

```powershell
set ALLOW_NON_WSL=1 && make dev-all
```

## Docker-based reproducible builds (Debian / Fedora)

We provide Dockerfiles in `contrib/` that install the system deps and run the build inside a clean environment. This is the recommended way to produce Linux artifacts for release.

Convenience targets & scripts:

Typical Docker commands (manual):

```bash
docker build -t check_vpn_debian -f contrib/Dockerfile.debian .
docker run --rm -v "$PWD":/work -w /work check_vpn_debian bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"
```

If Docker is installed but the daemon is not running, start Docker Desktop (Windows/macOS) or `sudo systemctl start docker` on Linux.

## CI notes (GitHub Actions)

The previous CI matrix workflow (`ci-matrix.yml`) has been archived. The
repository now uses simplified CI focused on Docker-based multi-architecture
builds and a single release workflow. Active workflows in `.github/workflows/`
include `docker-builds.yml` (push/PR builder) and `release-docker.yml` (tag
-triggered release for v0.5b).

- On `ubuntu-latest` the workflow uses Docker to produce Debian/Fedora artifacts.
- On `macos-latest` the workflow runs a native `cargo build --release`.
- On `windows-latest` the CI workflow runs Windows-native packaging steps to prepare Windows artifacts; local contributors should prefer WSL2 or CI for producing packaged Windows binaries.

The workflow uploads the built artifacts as release-ready tarballs or zip files.

### Running long/ignored tests in CI

The repository exposes a dedicated reproducible workflow that runs long or network-facing tests inside the Debian-based test container (`.github/workflows/integration-long.yml`). This workflow can be triggered manually and also runs on a weekly schedule; by default it will only run when a PR is labeled with `run-long-tests` (to avoid accidental long runs from every PR).

If you have a self-hosted runner prepared to run longer tests (for example, with elevated network access or more CPU), you can also drive a self-hosted job by either:

- Adding the `self-hosted-tests` label to the PR — the workflow will detect the label and run `long-integration-self-hosted` on a `self-hosted` runner (if available).
- Or manually dispatch the workflow with the `run_self_hosted=true` input via the Actions UI to run the self-hosted job explicitly.

This allows maintainers to run full-network and longer integration tests in a reproducible runtime while avoiding accidental, unnecessary long runs for routine PRs.

## Troubleshooting and common errors

- Docker CLI present but daemon not responding: start Docker Desktop (Windows/macOS) or systemd docker service on Linux.
- Missing native libraries (OpenSSL/libcurl): install the platform-specific dev packages listed above or enable Cargo features that vendor dependencies.
- On Windows, if `cl.exe` not found, install Visual Studio Build Tools C++ workload and ensure the developer command prompt environment is used or Visual Studio tools are on PATH.
- If a test interacts with the network and is marked `#[ignore]`, run ignored tests explicitly:

```bash
cargo test -- --ignored
```

Alternatively the repo includes a helper script to run the full test-suite (including ignored tests) inside the same reproducible Debian container used by CI:

```bash
# Build the test image and run ignored tests inside it
./contrib/ci docker
```

## Make and script shortcuts

- Use `make` on Unix or WSL to run targets defined in `Makefile` (requires GNU make).
- Windows users without `make`: use WSL2 and the POSIX helpers (for example `wsl ./scripts/docker-build.sh debian build-release` or run `wsl ./scripts/test-all-docker.sh`) — PowerShell helpers have been removed in favor of WSL2 and POSIX scripts.

## Files added by this repository to support these flows

- `Makefile` — convenience tasks for local dev and Docker builds (targets: `all`, `release`, `debian-build-release`, `fedora-build-release`, `test-all-docker`, `windows-test-all-docker`).
- `scripts/*.sh` — POSIX shell helpers for builds and Docker operations (recommended). WSL2 users can run these from a WSL shell or via `wsl` from a Windows shell.
- `scripts/*.sh` — POSIX shell counterparts for WSL/Linux/macOS.
.github/workflows/matrix.yml — previously used matrix workflow is archived in .github/workflows/archived/. To reproduce the CI workflows locally, use `./scripts/dev-all.sh`.

---

If you want I can further tighten this document: produce a checklist for each OS, or add one-line install snippets for popular package managers (apt, dnf, brew, choco). Which would you prefer?
