# check_vpn — Simple VPN connection watchdog

Small, fast, and reliable VPN connection monitor written in Rust. It checks your public ISP/ASN and takes configurable action if your VPN drops (restart a unit, run a script, or request a reboot).

## Features

- Automatic VPN monitoring via public provider APIs
- Flexible responses: restart a systemd unit, run a command, or reboot
- Small, dependency-minimal metrics endpoint for simple health checks
- Testable design, CI, and packaging-friendly layout

## Quick start

Install a pre-built binary (releases page):

```sh
sudo install -m 755 check_vpn /usr/local/bin/check_vpn
```

Build and install from source:

```sh
cargo build --release
sudo install -m 755 target/release/check_vpn /usr/local/bin/check_vpn
```

Create a config dir and copy the example configuration:

```sh
sudo mkdir -p /etc/check_vpn
sudo cp examples/check_vpn.xml /etc/check_vpn/config.xml
sudo chmod 644 /etc/check_vpn/config.xml
```

Edit `/etc/check_vpn/config.xml` to match your environment. Minimal example:

```xml
<config>
  <interval>60</interval>
  <isp_to_check>Your Real ISP Name</isp_to_check>
  <vpn_lost_action_type>restart-unit</vpn_lost_action_type>
  <vpn_lost_action_arg>openvpn-client@myvpn.service</vpn_lost_action_arg>
  <dry_run>false</dry_run>
</config>
```

### Examples

Run once (dry-run, no actions taken):

```sh
check_vpn --config-file /etc/check_vpn/config.xml --dry-run --run-once
```

Run continuously:

```sh
check_vpn --config-file /etc/check_vpn/config.xml
```

Restart a systemd unit when ISP is detected:

```sh
check_vpn -i "Your ISP Name" -t restart-unit -a "openvpn-client@myvpn.service"
```

Run a script when the ISP is detected:

```sh
check_vpn -i "Your ISP Name" -t command -a "/usr/local/bin/reconnect_vpn.sh"
```

## System service (systemd)

Create a service user and copy the example systemd units from `contrib/`.

```sh
sudo useradd --system --no-create-home --shell /usr/sbin/nologin checkvpn
sudo mkdir -p /etc/check_vpn /var/log/check_vpn
sudo chown checkvpn:checkvpn /etc/check_vpn /var/log/check_vpn
```

Then enable the provided service file and start the service:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
sudo systemctl status check_vpn
```

For desktop/personal runs, you can use a user unit instead of a system unit.

## Metrics & logs

If enabled, the metrics endpoint is available (default 127.0.0.1:9090). Logs are by default handled by systemd journald when running as a service.

## Troubleshooting

Check journal logs when a systemd service is used:

```sh
sudo journalctl -u check_vpn -f
```

If an action fails, try the action manually (e.g. `sudo systemctl restart your-vpn-unit`) and verify permissions.

## Contributing

See `CONTRIBUTING.md` for the project's contribution guidelines and `CODE_OF_CONDUCT.md` for community expectations.

## License

This project is licensed under the terms of the `LICENSE` file in the project root (MIT by default).

---

For more examples and advanced configuration, consult the repository `contrib/` and `docs/` directories.

## CI and releases


This repository uses a simplified CI pipeline. Pull requests and main-branch
pushes build multi-architecture Docker images only. Full test and linting
checks are expected to be run locally by contributors before pushing. Releases
are produced from the single canonical tag `v0.5b` which generates multi-arch
packages via Docker and uploads them to GitHub Releases.

Local macOS package (developer):

If you're on a macOS development box you can create a local system-level installer (.pkg) for testing.
This is intentionally local-only (no CI, no uploading) and produces a package you can inspect or install.

Steps:

- Build and create a .pkg and tarball (must run on macOS):

    ```bash
    make package-macos-local
    ```

    The script writes `release/macos/check_vpn-<version>.pkg` and `artifacts/macos/check_vpn-macos-<version>.tar.gz`.

  - Inspect or simulate an install (safe — does not modify host when using the simulate mode):

    ```bash
    make test-macos-pkg-local
    ```

Notes:

- By default the package's postinstall does NOT auto-load the LaunchDaemon (safe default). If you want the
  installer to try to load the LaunchDaemon automatically, set `CHECK_VPN_PKG_AUTOLOAD=true` when building (not recommended for casual testing).
- Signing and notarization are not included here; they require an Apple Developer account and are left as a follow-up.

## Running CI locally (replace GitHub Actions)

If you want to run the full CI and packaging locally without relying on GitHub Actions, the repo provides scripts and helpers to reproduce the CI tasks on your host and in containers:

- Run unit and integration tests locally (single-threaded):

```bash
RUST_TEST_THREADS=1 cargo test --workspace -- --ignored --nocapture
```

- Run the containerized test suite (Debian image):

```bash
./contrib/ci docker
```

- Run the developer local 'CI' tasks (host tests, docker tests and packaging):

```bash
./scripts/dev-all.sh
# Optional flags: --skip-docker --skip-packages --include-ignored
```

Note: On Windows this script uses a WSL2 guard and will not run in non-WSL shells by default. Use WSL2 (e.g., `wsl ./scripts/dev-all.sh`) or pass `--allow-non-wsl` to force execution.

Run `make help` for a short summary or one of the extended help commands below:

- `make help-dev` — development helpers
- `make help-docker` — docker test/build helpers
- `make help-packaging` — packaging helpers
- `make help-windows` — Windows/WSL guidance
- `make help-targets` — extended Makefile targets (includes `dev-all` and WSL guard info)

If you prefer to use the Makefile shortcut `make dev-all`, the Makefile includes a similar guard and will abort in a native Windows shell by default. To override that guard and force the target to run from a native Windows shell, set the environment variable `ALLOW_NON_WSL=1`:

```bash
ALLOW_NON_WSL=1 make dev-all
```

From PowerShell/CMD:

```powershell
set ALLOW_NON_WSL=1 && make dev-all
```

## WSL2 setup (Windows)

If you're on Windows, install WSL2 and run the POSIX helper scripts inside the WSL2 environment. For a quick setup:

- Install WSL2: follow the Microsoft guide: [Install WSL2](https://learn.microsoft.com/windows/wsl/install).
- Install a WSL distro like Ubuntu from the Microsoft Store and enable Docker Desktop's WSL integration if needed.

Install Rust in WSL via `rustup` (see [rustup](https://rustup.rs)):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Once WSL and Rust are installed, use the dev helper from WSL:

```bash
wsl ./scripts/dev-all.sh
```


On Windows, use WSL2 and run the POSIX helper from inside WSL rather than PowerShell: `wsl ./scripts/dev-all.sh` (Optional flags: `--skip-docker`, `--skip-packages`, `--include-ignored`)

These scripts are intended to reproduce the primary CI tasks locally: running tests, building Docker images, and producing packaging artifacts. When running packaging steps locally you will need Docker/Buildx and QEMU or platform-specific packaging tools (e.g., cargo-deb or fpm) installed and available.
