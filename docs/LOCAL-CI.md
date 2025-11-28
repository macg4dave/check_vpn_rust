# Running CI workflows locally (Developer Guide)

This repository previously used GitHub Actions for CI. The workflows have
been archived, and this page documents how to reproduce CI tasks locally
on your machine. The provided scripts are intended to be a one-command
developer-friendly replacement for CI tasks.

## Prerequisites

- Docker (Desktop or engine) when running Docker-based tests or packaging.
- `docker buildx` and `docker run --privileged --rm tonistiigi/binfmt --install all` to use QEMU emulation for multi-arch builds.
- Optional: `cargo-deb` (for Debian packaging) and `fpm` (for RPM packaging) if you prefer local packaging without Docker.

## WSL2 & Rust (Windows guidance)

- Install WSL2: follow the official Microsoft guide: [Install WSL2](https://learn.microsoft.com/windows/wsl/install).
- Install a Linux distribution (Ubuntu recommended). Enable Docker Desktop WSL integration if you plan to run Docker in WSL.
- Install Rust inside WSL (via rustup): [rustup](https://rustup.rs) — run `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh` inside your WSL shell.
- After installing WSL2 and Rust inside WSL, run the POSIX helpers from within your WSL shell:
  - `./scripts/dev-all.sh` or `./scripts/test-all-docker.sh` as needed.

Note: The `make dev-all` target includes a guard that prevents running the target from a native Windows shell (PowerShell/CMD/Git Bash) by default; this reduces accidental mismatched environments. To override the Makefile guard (not recommended), set `ALLOW_NON_WSL=1`:

```bash
ALLOW_NON_WSL=1 make dev-all
```

From PowerShell/CMD:

```powershell
set ALLOW_NON_WSL=1 && make dev-all
```

## Script summary

  1) Host tests (unit & integration), single-threaded
  2) Containerized tests (Debian/Fedora) using `contrib/ci docker`
  3) Packaging using `./scripts/build-all-packages.sh` (Docker based)


 
Run all host and Docker tests and packaging (POSIX):

```bash
./scripts/dev-all.sh
```

## Usage examples

Run tests but skip packaging (to save time):

```bash
./scripts/dev-all.sh --skip-packages
```

Run host tests & Docker tests from Windows (WSL2):

```bash
wsl ./scripts/dev-all.sh
```

Note: on Windows, this script will refuse to run in non-WSL shells by default; use a WSL2 shell (e.g. `wsl ./scripts/dev-all.sh`) or pass `--allow-non-wsl` to force it. If you call the Makefile shortcut `make dev-all`, the Makefile guard is controlled via the `ALLOW_NON_WSL` environment variable (e.g. `ALLOW_NON_WSL=1 make dev-all`).

## Notes


If you have suggestions for extending the local dev scripts (e.g., more robust Windows variant, optional npm or other environment setup), open a PR or an issue to discuss.
