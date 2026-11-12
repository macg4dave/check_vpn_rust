# Developing check_vpn

Version 1.1.0 supports native Windows, Linux and macOS builds. Rust 1.93.1 or
newer is required. Use rustup with rustfmt and Clippy. Linux builds use native
TLS and need pkg-config and OpenSSL development headers; Windows needs the
Visual Studio C++ build tools. Bash scripts require Bash and ShellCheck
(`apt install shellcheck`, `dnf install ShellCheck`, or `brew install shellcheck`).

## Local checks

Run from the repository root:

```sh
cargo fmt --all -- --check
cargo lint
cargo lint-all
cargo lint-minimal
cargo test --workspace --locked
cargo test-all --locked
cargo test-minimal --locked
cargo test --workspace --doc --all-features --locked
bash scripts/check-shell.sh
```

`make check` runs formatting, all-feature Clippy, shell checks and regressions, and default Rust
tests. `make scriptcheck` runs Bash syntax checks and ShellCheck over scripts,
package lifecycle scripts, and Git hooks. It fails if ShellCheck is unavailable;
it does not silently skip findings. `bash tests/shell_regressions.sh` checks the
packaging and hook behaviors with isolated fixtures.

Store captured output under `logs/`, which is ignored by Git and Docker:

```sh
mkdir -p logs
cargo test --workspace --locked > logs/tests.log 2>&1
```

Use local mock servers and injected action runners for tests. Real-network tests
are ignored by default; select them explicitly only in a suitable environment.
Serialize tests that mutate shared environment variables and restore the previous
values. A restoration guard alone does not prevent concurrent access.

To enable the optional pre-commit hook, run `bash scripts/install-hooks.sh`.
The hook scans staged files for credentials, checks shell scripts, and runs Rust
checks without staging or modifying files. Do not overwrite a custom hook without
reviewing it.

## Build and package

```sh
cargo build --release --locked
./target/release/check_vpn --version
bash scripts/build-all-packages.sh --help
```

The Cargo package version is the default for Debian, RPM, macOS, and Windows
artifacts. A stable `X.Y.Z` override can be supplied where documented. Package
architecture comes from the build and package tooling; renaming an artifact must
never be used to change its architecture.

Debian and Fedora packages use Docker and the helpers in `contrib/packaging/`.
`make package-all`, `make package-debian`, and `make package-fedora` run the
packaging orchestrator. Outputs go under `release/`. Local packaging without
Docker requires cargo-deb or fpm (`--no-docker`). Multi-architecture builds need
a working Docker buildx builder and emulation for non-native architectures.

On Windows, run `scripts/build-windows-package.ps1` in PowerShell. It builds the
release binary and puts a versioned ZIP under `artifacts/windows/`, containing
only the executable, sample configuration, and README. On macOS,
`make package-macos-local` creates a `.pkg` and tarball; loading the LaunchDaemon
is an explicit build-time opt-in using `CHECK_VPN_PKG_AUTOLOAD=true`.

Use `scripts/test-macos-pkg.sh inspect <package>` to inspect macOS packages.
The `simulate` mode runs the macOS installer against a mounted disk image and
may require sudo; use a disposable machine for install testing.

## Containers and virtual machines

```sh
make debian-test
make fedora-test
bash scripts/test-all-docker.sh
bash scripts/docker-build.sh debian build-release
```

The test images contain integration tests and their fixtures. The Debian image
uses a pinned official Rust image; the Fedora image installs stable Rust.
`docker-build.sh` stores mounted Linux build output in `target-debian/` or
`target-fedora/`, separately from the native host's `target/` directory.
The `.dockerignore` excludes generated artifacts from the build context.

For systemd integration, `bash contrib/run_systemd_container.sh` creates a
privileged, systemd-capable test container. Use it only for system integration
on a disposable host. Alternatively, `vagrant up` provisions the Debian VM in
`Vagrantfile`, builds the release, and installs it inside that VM.

Installation and recovery permissions are documented in [INSTALL.md](INSTALL.md).
No GitHub Actions workflows are installed; run and review checks locally before
preparing a release. See [AGENTS.md](AGENTS.md) for the full Rust engineering policy.
