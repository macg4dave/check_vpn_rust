# check_vpn — Simple VPN connection watchdog

Small, fast, and reliable VPN connection monitor written in Rust. It checks your public ISP/ASN and takes configurable action if your VPN drops (restart a unit, run a script, or request a reboot).

## Features

- Automatic VPN monitoring via public provider APIs
- Flexible responses: restart a systemd unit, run a command, or reboot
- Built-in providers default to HTTPS for ISP lookups (ip-api.com) with override hooks
- Config validation ensures at least one provider is enabled/configured to avoid silent failures
- Provider fallback, configuration reloads and bounded recovery commands
- Testable design and packaging-friendly layout

Version **1.1.0** requires Rust **1.93.1+** to build. See [DEVELOPING.md](DEVELOPING.md) for checks and packaging.

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
check_vpn --config /etc/check_vpn/config.xml --dry-run --run-once
```

Run continuously:

```sh
check_vpn --config /etc/check_vpn/config.xml
```

Restart a systemd unit when ISP is detected:

```sh
check_vpn -i "Your ISP Name" -t restart-unit -a "openvpn-client@myvpn.service"
```

Run a script when the ISP is detected:

```sh
check_vpn -i "Your ISP Name" -t command -a "/usr/local/bin/reconnect_vpn.sh"
```

Commands are executed without a shell; quoting is respected (parsed with shell-words) and obvious shell metacharacters are rejected. Example with args:

```sh
check_vpn -t command -a '/usr/local/bin/reconnect_vpn.sh --reason "vpn lost"'
```

Provider configuration: ensure at least one provider is enabled (default ip-api + ifconfig.co, or custom URLs). Validation fails fast if all providers are disabled or missing to avoid silent loops.

## Monitoring behaviour

An exact match to `isp_to_check` or any explicitly configured `isp_aliases` means the VPN is lost. Any other nonempty ISP means VPN active; the VPN provider's ISP does not need to be known. Failed or blank lookups are errors, not active-VPN results. Different services may name your normal ISP differently: add each exact name as an alias (`--isp-alias "Normal ASN Organisation"`).

Providers run before TCP connectivity diagnostics. A failed TCP probe never prevents a working provider from detecting your normal ISP. No recovery action is taken solely because the connection is offline.

The default action is `log`, a portable log message. Configure `restart-unit`, `command` or `reboot` explicitly when recovery is wanted. Recovery attempts have a 60-second cooldown after completion, including failed attempts (`--recovery-cooldown`). Commands have a 30-second timeout (`--action-timeout`). Checks wait the configured interval after completing; work duration adds to the start-to-start interval.

Daemon mode logs lookup and recovery errors and continues unless `--exit-on-error` is set. `--run-once` always reports these failures through its exit status. Exit codes: 0 success, 1 lookup/runtime failure, 2 invalid configuration or CLI usage, 3 recovery action failure. A successful run-once recovery still exits 0.

The configuration source is resolved once at startup. Reloads use that same file; missing or invalid replacements retain the previous settings, or exit with code 2 under `--strict-config`. Starting without a file uses defaults until restart. No configuration is automatically written during discovery; use `--init` to create one.

List items in XML use repeated field tags, matching generated configuration:

```xml
<isp_aliases>Normal ISP alternate name</isp_aliases>
<connectivity_endpoints>8.8.8.8</connectivity_endpoints>
<connectivity_endpoints>1.1.1.1</connectivity_endpoints>
<connectivity_ports>443</connectivity_ports>
<connectivity_ports>53</connectivity_ports>
<recovery_cooldown_secs>60</recovery_cooldown_secs>
<action_timeout_secs>30</action_timeout_secs>
```

For ordered custom JSON providers, repeat `<providers>` entries containing `<url>` and optional `<key>`:

```xml
<providers><url>https://example.com/identity</url><key>org</key></providers>
```

Legacy `custom_json_server` (first) and `provider_urls` remain supported. The complete order is custom server, `providers`, legacy URLs, ip-api, ifconfig.co. HTTP clients are reused across checks. Provider lookup has a total 30-second budget and a 10-second budget per provider, each HTTP request is bounded by 5 seconds, and failure diagnostics have a separate 5-second budget. Shutdown stops retries and prevents later actions; an in-flight HTTP request may take up to its timeout to finish.

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

For desktop/personal runs, use the dedicated user unit in `contrib/check_vpn-user.service`.

The service account needs explicit authorization to restart a system VPN unit. Follow [Recovery permissions](INSTALL.md#recovery-permissions); the example rule allows only restarting one named VPN unit. Packages default to log-only recovery and do not install a privilege-granting rule automatically.

## Troubleshooting

Check journal logs when a systemd service is used:

```sh
sudo journalctl -u check_vpn -f
```

If an action fails, try the action manually (e.g. `sudo systemctl restart your-vpn-unit`) and verify permissions.

## Windows notes

- The binary builds and runs on Windows 10/11. systemd/D-Bus actions are Linux-only; on Windows prefer the `command` action to run a recovery script.
- Use `--config C:\path\to\config.xml` (or `CHECK_VPN_CONFIG`) and run from PowerShell or Task Scheduler. Hot-reload of the config file is supported when the service stays running.
- Connectivity checks and provider lookups are identical to Linux; ensure outbound HTTPS is allowed through your firewall.

## Contributing

See `CONTRIBUTING.md` for the project's contribution guidelines and `CODE_OF_CONDUCT.md` for community expectations.

## License

This project is licensed under the terms of the `LICENSE` file in the project root (MIT by default).

---

For more examples and advanced configuration, consult the repository `contrib/` and `docs/` directories.

## CI and releases

The old GitHub Actions workflows have been removed. Run checks locally:
`cargo fmt --all -- --check`, `cargo lint-all`, and `cargo test-all`.
See `AGENTS.md` for default and minimal-feature checks. Packaging helpers remain
available for local builds.

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
