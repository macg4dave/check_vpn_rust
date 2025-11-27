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
