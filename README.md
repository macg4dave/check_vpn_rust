# check_vpn (Rust)

[![CI](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml)

This repository contains a small Rust utility that checks if a VPN appears to be lost by checking the public ISP reported by ip-api.com. If the ISP matches a configured value (the ISP used when VPN is lost), the program runs an action such as rebooting or reconnecting the VPN.

Build

You need Rust and cargo installed. Then:

```sh
cargo build --release
```

# check_vpn (Rust)

[![CI](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml)

A small Rust utility that detects when your VPN appears to be lost (by checking the public ISP reported by an IP lookup service) and performs a configurable action such as restarting a unit, running a command, or rebooting.

This README provides build/install instructions, full usage examples, and systemd unit examples for Debian and Fedora-style systems.

## Quick start

Build from source (requires Rust/cargo):

```sh
cargo build --release
# resulting binary: target/release/check_vpn
```

Run a quick dry-run to verify behavior (no actions executed):

```sh
target/release/check_vpn --dry-run
```

Run a single iteration (useful for testing/config verification):

```sh
target/release/check_vpn --run-once --isp-to-check "MyISP" --dry-run
```

## Usage (flags)

Key command-line flags (the project uses `clap`):

- `--interval <seconds>` — seconds between checks (overrides config)
- `-i, --isp-to-check <STRING>` — ISP string that indicates VPN is lost
- `-t, --vpn-lost-action-type <reboot|restart-unit|command>` — action type
- `-a, --vpn-lost-action-arg <ARG>` — argument for the action type (unit name or command)
- `--dry-run` — log the action instead of executing it
- `--config-file <PATH>` — load configuration from an XML file (see `examples/check_vpn.xml`)
- `--connectivity-endpoint <HOST|IP>` — connectivity probe endpoints (repeatable / CSV)
- `--connectivity-ports <PORTS>` — ports to try when endpoints don't include a port
- `--connectivity-timeout <secs>` — timeout for connectivity checks
- `--connectivity-retries <n>` — number of retries for connectivity checks
- `--run-once` — execute a single iteration and exit
- `-v, --verbose` — increase logging verbosity (repeatable)
- `--enable-metrics` — enable a small HTTP health/metrics endpoint
- `--metrics-addr <ADDR:PORT>` — address for metrics endpoint (default `0.0.0.0:9090`)
- `--exit-on-error` — exit with non-zero codes on errors even in long-running mode

Use `--help` to see the full set of flags and defaults.

## Example command lines

# Basic continuous run with default config
check_vpn --enable-metrics --metrics-addr 127.0.0.1:9090

# Run once for manual testing (dry-run)
check_vpn --run-once --isp-to-check "YourISP" --dry-run

# Use command action to call a custom script on VPN lost
check_vpn -t command -a "/usr/local/bin/reconnect_vpn.sh"

# Restart a systemd unit when VPN is lost
check_vpn -t restart-unit -a "openvpn-client@myvpn.service"

## Configuration

The application can read configuration from an XML file (example in `examples/check_vpn.xml`) or via CLI flags. The config file location can be overridden with `--config-file`.

Reasonable default runtime paths (packaging/installation should follow):

- Config: `/etc/check_vpn/config.xml` (directory `/etc/check_vpn/`)
- Logs (if writing to file): `/var/log/check_vpn/check_vpn.log`
- Binary (manual install): `/usr/local/bin/check_vpn` or `/usr/bin/check_vpn` (packaging decides)

Make sure directories exist and are owned by the service user when running as a non-root service.

## Metrics & health endpoint

Enable with `--enable-metrics`. By default the server binds to `0.0.0.0:9090`. It exposes:

- `/health` — returns HTTP 200 when the process is running (liveness/readiness)
- `/metrics` — a plain-text metrics payload suitable for simple scraping

Examples:

```sh
curl -sS http://127.0.0.1:9090/health
curl -sS http://127.0.0.1:9090/metrics
```

## Systemd service examples

Below are two systemd unit examples: one for a system service (runs as `checkvpn` user) and one for a simple user service.

Notes before enabling:

- Create a dedicated user/group if you don't want the service to run as `root`:

```sh
sudo useradd --system --no-create-home --group nogroup --shell /usr/sbin/nologin checkvpn || true
sudo mkdir -p /etc/check_vpn /var/log/check_vpn
sudo chown checkvpn:checkvpn /etc/check_vpn /var/log/check_vpn
```

- Install the binary to `/usr/local/bin/check_vpn` or `/usr/bin/check_vpn`.

System service (recommended for servers):

`/etc/systemd/system/check_vpn.service`:

```ini
[Unit]
Description=check_vpn - VPN monitoring and auto-reconnect
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=checkvpn
Group=checkvpn
ExecStart=/usr/local/bin/check_vpn --config-file /etc/check_vpn/config.xml --enable-metrics --metrics-addr 127.0.0.1:9090
Restart=on-failure
RestartSec=10
RuntimeDirectory=check_vpn
# If you write logs to /var/log/check_vpn, ensure permissions are correct

[Install]
WantedBy=multi-user.target
```

Enable and start:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
sudo journalctl -u check_vpn -f
```

User service (per-user, placed under `~/.config/systemd/user/`):

`~/.config/systemd/user/check_vpn.service`:

```ini
[Unit]
Description=check_vpn (user)
After=network-online.target

[Service]
Type=simple
ExecStart=/home/youruser/.local/bin/check_vpn --config-file /home/youruser/.config/check_vpn/config.xml --enable-metrics
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

Start with:

```sh
systemctl --user daemon-reload
systemctl --user enable --now check_vpn.service
journalctl --user -u check_vpn -f
```

## Log rotation

If your service writes to `/var/log/check_vpn/check_vpn.log`, add a logrotate config at `/etc/logrotate.d/check_vpn`:

```
/var/log/check_vpn/check_vpn.log {
	copytruncate
	daily
	rotate 14
	compress
	missingok
	notifempty
}
```

If you rely on `journald` (the default when using systemd without explicit file logs), journal rotation is managed by systemd and `journald.conf`.

## Fedora (SELinux) notes

On Fedora with SELinux Enforcing enabled, if you install the binary to a non-standard path or the service needs to access network resources/files with different contexts, you may need to:

- Ensure executable has correct context: `sudo restorecon -v /usr/local/bin/check_vpn`
- If you write to `/var/log/check_vpn`, ensure correct context or use `chcon`/`semanage fcontext` to persist changes:

```sh
sudo semanage fcontext -a -t var_log_t "/var/log/check_vpn(/.*)?"
sudo restorecon -Rv /var/log/check_vpn
```

If the service is prevented from performing needed network operations, examine `ausearch -m avc -ts recent` and create a simple local policy or adjust booleans (be conservative):

```sh
sudo ausearch -m avc -ts today | audit2allow -M check_vpn_local
sudo semodule -i check_vpn_local.pp
```

## Packaging notes (Debian / Fedora)

- Debian: building a .deb can be done with `cargo deb` or packaging into a proper Debian package that installs the binary to `/usr/bin`, puts config into `/etc/check_vpn/` and installs the systemd unit.
- Fedora: build an RPM that installs the binary and unit. For SELinux-managed systems ensure file contexts are correct or ship a policy module.

Minimal packaging checklist:

1. Binary -> `/usr/bin/check_vpn`
2. Config template -> `/etc/check_vpn/config.xml` (owner root:root mode 0644)
3. Systemd unit -> `/lib/systemd/system/check_vpn.service` (or `/etc/systemd/system/` for local installs)
4. Post-install enable the unit (packagers typically avoid auto-start in some distros; include instructions)
5. Logrotate file (optional) -> `/etc/logrotate.d/check_vpn`

## Testing

Run unit & mocked tests (default):

```sh
cargo test
```

Run ignored integration tests (real-network tests) explicitly:

```sh
cargo test -- --ignored
```

There are integration tests in `tests/` that are ignored by default. Use `-- --ignored` to run them.

## Troubleshooting

- If the service won't start, check `sudo journalctl -u check_vpn -b`.
- If the program can't determine ISP, run `--run-once --dry-run --verbose` to see more debug output.
- If file permissions prevent reading config or writing logs, ensure the `checkvpn` user has access to `/etc/check_vpn` and `/var/log/check_vpn`.

## Contributing

Contributions welcome. Please open issues for bugs or feature requests and consider sending pull requests with tests.

## License

See the repository `LICENSE` file for license details.
