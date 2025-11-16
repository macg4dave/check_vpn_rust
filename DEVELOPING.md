# Developing and building check_vpn (platform guide)

This file describes how to build and test this repository across supported OSes: macOS, Debian/Fedora (via Docker), and Windows. It also describes the CI approach used by the project.

Summary
- Supported platforms: macOS, Debian, Fedora (packaged via Docker), Windows (native / WSL2)
- CI: GitHub Actions matrix — macOS builds run on `macos-latest`, Debian/Fedora built inside Docker on `ubuntu-latest`, Windows builds run on `windows-latest` using PowerShell helper.

Prerequisites
- Rust toolchain: install rustup from https://rustup.rs
- Docker Desktop (for Docker-based Debian/Fedora builds)
- (Windows native) Visual Studio Build Tools (C++ workload) if using the MSVC toolchain
- WSL2 (optional, recommended for parity with Linux)

Quick local commands

Native development (macOS/Windows/Linux)

From the repository root:

```bash
# Run the program during development
cargo run

# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features
```

Windows (PowerShell)

Use the provided PowerShell helper to build and package a Windows binary:

```powershell
# Native Windows (MSVC recommended)
# From repo root (PowerShell)
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-windows.ps1 -Release
```

If you prefer WSL2 for parity with Debian builds, open WSL and use the Linux commands below.

Docker-based Debian/Fedora builds (recommended for reproducible Linux artifacts)

These use the Dockerfiles under `contrib/`.

From PowerShell (convenience scripts):

```powershell
# Debian
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-debian-docker.ps1

# Fedora
powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build-fedora-docker.ps1
```

From a Linux shell (WSL or native):

```bash
# Debian
docker build -t check_vpn_debian -f contrib/Dockerfile.debian .
docker run --rm -v "$PWD":/work -w /work check_vpn_debian \
  bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"

# Fedora
docker build -t check_vpn_fedora -f contrib/Dockerfile.fedora .
docker run --rm -v "$PWD":/work -w /work check_vpn_fedora \
  bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-fedora.tar.gz -C target/release check_vpn* || true"
```

macOS builds

- Recommended: use GitHub Actions macOS runner (`macos-latest`) to produce macOS artifacts. See `.github/workflows/ci-matrix.yml`.
- Alternative: build natively on a Mac using `cargo build --release`.
- Cross-compiling to macOS (from Linux/Windows) is not recommended unless you have an Apple SDK and osxcross; Apple SDKs are restricted by licensing and not bundled.

CI behaviour

- `ci-matrix.yml` (GitHub Actions) runs a matrix across `ubuntu-latest`, `macos-latest`, and `windows-latest`.
- On `ubuntu-latest`, the workflow builds Docker images from `contrib/Dockerfile.debian` and `contrib/Dockerfile.fedora`, runs `cargo build --release` inside them, and uploads tarball artifacts.
- On `macos-latest`, the workflow runs a native `cargo build --release` and uploads the resulting binary.
- On `windows-latest`, the workflow uses the `scripts/build-windows.ps1` helper to build and package a zip.

Troubleshooting

- If builds fail due to missing native dependencies (libssl, curl system libs), make sure those are installed in the Dockerfiles or on the host. The `contrib/Dockerfile.*` files should already install system deps — inspect them and add missing packages if needed.
- On Windows, prefer MSVC toolchain: `rustup default stable-x86_64-pc-windows-msvc` and install Visual Studio Build Tools.
- If you need to test Debian packaging or systemd integration, run inside the Debian Docker image so the environment matches the target.

Cleaning and packaging

- Use `cargo clean` to remove build artifacts.
- The PowerShell scripts and Docker helper steps produce `.tar.gz` or `.zip` artifacts under `target/release/`.

If you want, I can:
- Tidy up the `.github/workflows/` directory and keep only the chosen workflows (I left a small inert placeholder file so the old CI doesn't accidentally run).  
- Add a `Makefile` shortcut to call the PowerShell Docker wrappers from WSL-friendly paths.

---

Thanks — let me know which follow-up you want me to do next (clean old workflow entirely, add more packaging steps, or push a release automation).
