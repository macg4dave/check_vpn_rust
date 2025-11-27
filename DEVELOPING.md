# Developing and building check_vpn (platform guide)

This file describes how to build and test this repository across supported OSes: macOS, Debian/Fedora (via Docker), and Windows. It also describes the CI approach used by the project.

Summary
- Supported platforms: macOS, Debian, Fedora (packaged via Docker), Windows (native / WSL2)
- CI: GitHub Actions matrix — macOS builds run on `macos-latest`, Debian/Fedora built inside Docker on `ubuntu-latest`, Windows builds run on `windows-latest` using native Windows steps; locally prefer WSL2 or CI for packaged Windows artifacts.

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

### Debug / tracing

For deeper troubleshooting during development you can build and run with backtraces and the optional `observability` feature which enables `tracing` and structured logs:

```bash
# Quick helper (runs with RUST_BACKTRACE=1 and RUST_LOG=trace)
make debug

# Or run directly:
RUST_BACKTRACE=1 RUST_LOG=trace cargo run --features "observability" -- --run-once --dry-run -vv
```
Testing notes
----------------
- The repository includes lightweight integration test helpers in `tests/support/mod.rs`.
  Use `support::ScopedEnv::set_str(key, value)` in integration tests to temporarily set
  environment variables and automatically restore the previous state when the guard
  is dropped. This avoids unsafe env mutation blocks and reduces flakiness in parallel
  test runs.

 - For unit tests that mutate global state (for example, `RUST_LOG`), prefer serializing
   tests with `serial_test::serial` or use a local RAII guard to restore the environment.
# Format and lint
cargo fmt --all
cargo clippy --all-targets --all-features
```

Windows / WSL2

If you are developing on Windows we recommend using WSL2 for parity with the Debian/Fedora build environments and for compatibility with the POSIX helpers in `./scripts`.

- To use WSL2 from Windows: `wsl make release` or `wsl ./scripts/docker-build.sh debian build-release`
- You can also open a WSL shell and run the POSIX scripts directly: `./scripts/test-all-docker.sh`, `./scripts/docker-build.sh` or `./scripts/build-all.sh`.

Docker-based Debian/Fedora builds (recommended for reproducible Linux artifacts)

These use the Dockerfiles under `contrib/`.

From a POSIX shell (WSL or native):

```bash
# Debian
./scripts/docker-build.sh debian build-release

# Fedora
./scripts/docker-build.sh fedora build-release
```

From a Linux shell (WSL or native):

```bash
# Debian
docker build -t check_vpn_debian -f contrib/Dockerfile.debian .
docker run --rm -v "$PWD":/work -w /work check_vpn_debian \
  bash -lc "cargo build --release && tar -czf /work/target/release/checkvpn-debian.tar.gz -C target/release check_vpn* || true"

Note: The Dockerfile and helper scripts send the repository as the build context (COPY . .). If your workspace contains large build artifacts like `target/` the build context size can exceed 20GB and Docker BuildKit or the daemon may fail. To avoid this:

- Ensure a `.dockerignore` exists at the repository root which excludes `target/` and other large directories.
- If for some reason you need to include large files, you can bypass the safety check by setting `DOCKER_BUILD_ALLOW_LARGE=1` in your environment before running the helper scripts.

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
On `windows-latest`, CI runs Windows-native packaging steps on the `windows-latest` runner; locally we recommend using WSL2 or CI for packaged Windows artifacts.

Troubleshooting

- If builds fail due to missing native dependencies (libssl, curl system libs), make sure those are installed in the Dockerfiles or on the host. The `contrib/Dockerfile.*` files should already install system deps — inspect them and add missing packages if needed.
- On Windows, prefer MSVC toolchain: `rustup default stable-x86_64-pc-windows-msvc` and install Visual Studio Build Tools.
- If you need to test Debian packaging or systemd integration, run inside the Debian Docker image so the environment matches the target.

Cleaning and packaging

- Use `cargo clean` to remove build artifacts.
- The helper scripts and Docker wrapper steps produce `.tar.gz` or `.zip` artifacts under `target/release/`.

If you want, I can:
- Tidy up the `.github/workflows/` directory and keep only the chosen workflows (I left a small inert placeholder file so the old CI doesn't accidentally run).  
- Add a `Makefile` shortcut to call the POSIX Docker helpers from WSL-friendly paths (for example `wsl ./scripts/docker-build.sh ...`).

---

Thanks — let me know which follow-up you want me to do next (clean old workflow entirely, add more packaging steps, or push a release automation).
