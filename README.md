# check_vpn

[![CI](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml)

**Automatic VPN monitoring and reconnection for Linux systems**

`check_vpn` is a lightweight Rust utility that continuously monitors your VPN connection by checking your public ISP. When it detects your real ISP (indicating VPN disconnection), it automatically takes action to restore your connection—whether that's restarting your VPN service, running a custom reconnection script, or rebooting the system.

### Key Features

- 🔒 **Automatic VPN monitoring** - Continuously checks your public ISP to detect VPN drops
- ⚡ **Fast and lightweight** - Written in Rust for minimal resource usage
- 🔧 **Flexible actions** - Restart systemd units, run custom commands, or reboot
- 📊 **Built-in metrics** - HTTP endpoint for health checks and monitoring
- 🐧 **Linux-optimized** - Systemd integration, logrotate support, SELinux compatibility
- 🎯 **Easy configuration** - XML config file or command-line flags

Perfect for headless servers, always-on VPN clients, or any system where VPN uptime is critical.

---

## Installation

### Quick Install (Pre-built Binary)

Download the latest release from the [releases page](https://github.com/macg4dave/check_vpn_rust/releases) and install:

```sh
sudo install -m 755 check_vpn /usr/local/bin/check_vpn
```

### Build from Source

Requires Rust 1.70+ and cargo:

```sh
cargo build --release
sudo install -m 755 target/release/check_vpn /usr/local/bin/check_vpn
```

### Set Up Configuration

Create the configuration directory and copy the example config:

```sh
sudo mkdir -p /etc/check_vpn
sudo cp examples/check_vpn.xml /etc/check_vpn/config.xml
sudo chmod 644 /etc/check_vpn/config.xml
```

Edit `/etc/check_vpn/config.xml` to match your setup:

```xml
<?xml version="1.0" encoding="utf-8"?>
<config>
  <interval>60</interval>
  <isp_to_check>Your Real ISP Name</isp_to_check>
  <vpn_lost_action_type>restart-unit</vpn_lost_action_type>
  <vpn_lost_action_arg>openvpn-client@myvpn.service</vpn_lost_action_arg>
  <dry_run>false</dry_run>
</config>
```

---

## Usage

### Quick Start

Test your configuration with a dry run (no actions executed):

```sh
check_vpn --config-file /etc/check_vpn/config.xml --dry-run --run-once
```

Run continuously with metrics enabled:

```sh
check_vpn --config-file /etc/check_vpn/config.xml --enable-metrics
```

### Command-Line Options

### Command-Line Options

Key flags (use `check_vpn --help` for complete list):

**Basic Options:**
- `--config-file <PATH>` — Load configuration from XML file (default: searches common paths)
- `--interval <seconds>` — Seconds between VPN checks (default: 60)
- `-i, --isp-to-check <STRING>` — ISP name that indicates VPN is lost
- `--dry-run` — Log actions without executing them (safe testing mode)
- `--run-once` — Execute a single check and exit
- `-v, --verbose` — Increase logging verbosity (repeatable: -v, -vv, -vvv)

**Action Configuration:**
- `-t, --vpn-lost-action-type <TYPE>` — Action type: `reboot`, `restart-unit`, or `command`
- `-a, --vpn-lost-action-arg <ARG>` — Action argument (systemd unit name or command to run)

**Connectivity Options:**
- `--connectivity-endpoint <HOST|IP>` — Connectivity probe endpoints (repeatable)
- `--connectivity-ports <PORTS>` — Ports to try for connectivity checks
- `--connectivity-timeout <secs>` — Timeout for connectivity checks
- `--connectivity-retries <n>` — Number of retry attempts

**Metrics & Monitoring:**
- `--enable-metrics` — Enable HTTP metrics/health endpoint
- `--metrics-addr <ADDR:PORT>` — Metrics server address (default: `0.0.0.0:9090`)
- `--exit-on-error` — Exit with error code on failures (useful for systemd restarts)

### Usage Examples

**Monitor VPN and restart systemd unit when lost:**
```sh
check_vpn -i "Your ISP Name" -t restart-unit -a "openvpn-client@myvpn.service"
```

**Run custom reconnection script:**
```sh
check_vpn -i "Your ISP Name" -t command -a "/usr/local/bin/reconnect_vpn.sh"
```

**Test configuration without taking action:**
```sh
check_vpn --config-file /etc/check_vpn/config.xml --dry-run --run-once -vv
```

**Run with metrics enabled for monitoring:**
```sh
check_vpn --config-file /etc/check_vpn/config.xml --enable-metrics --metrics-addr 127.0.0.1:9090
```

---

## Systemd Integration

### System Service Setup

Create a dedicated service user (recommended for security):

```sh
sudo useradd --system --no-create-home --shell /usr/sbin/nologin checkvpn
sudo mkdir -p /etc/check_vpn /var/log/check_vpn
sudo chown checkvpn:checkvpn /etc/check_vpn /var/log/check_vpn
```

Create the systemd service file at `/etc/systemd/system/check_vpn.service`:

```ini
[Unit]
Description=VPN Connection Monitor
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

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
sudo systemctl status check_vpn
sudo journalctl -u check_vpn -f
```

### User Service (Optional)

For per-user installations, create `~/.config/systemd/user/check_vpn.service`:

```ini
[Unit]
Description=VPN Connection Monitor (User)
After=network-online.target

[Service]
Type=simple
ExecStart=%h/.local/bin/check_vpn --config-file %h/.config/check_vpn/config.xml --enable-metrics
Restart=on-failure
RestartSec=10

[Install]
WantedBy=default.target
```

Enable with:

```sh
systemctl --user daemon-reload
systemctl --user enable --now check_vpn.service
journalctl --user -u check_vpn -f
```

---

## Monitoring & Metrics

When metrics are enabled with `--enable-metrics`, check_vpn exposes an HTTP endpoint (default: `http://0.0.0.0:9090`).

### Available Endpoints

**Health Check:**
```sh
curl http://127.0.0.1:9090/health
# Returns: HTTP 200 OK when service is running
```

**Metrics:**
```sh
curl http://127.0.0.1:9090/metrics
# Returns: Plain-text metrics for monitoring systems
```

Use these endpoints with monitoring tools like Prometheus, Nagios, or simple HTTP monitoring scripts.

---

## Log Management

### Logrotate Configuration

If writing to `/var/log/check_vpn/check_vpn.log`, create `/etc/logrotate.d/check_vpn`:

```
/var/log/check_vpn/check_vpn.log {
    daily
    rotate 14
    compress
    missingok
    notifempty
    copytruncate
}
```

### Journald Logs

When running as a systemd service (recommended), logs are managed by journald:

```sh
# View recent logs
sudo journalctl -u check_vpn -n 100

# Follow logs in real-time
sudo journalctl -u check_vpn -f

# View logs since last boot
sudo journalctl -u check_vpn -b
```

---

## Platform-Specific Notes

### Fedora / RHEL / Rocky Linux (SELinux)

If SELinux is in Enforcing mode, ensure proper file contexts:

```sh
# Set correct context for binary
sudo restorecon -v /usr/local/bin/check_vpn

# Set context for log directory
sudo semanage fcontext -a -t var_log_t "/var/log/check_vpn(/.*)?"
sudo restorecon -Rv /var/log/check_vpn
```

If SELinux blocks operations, check audit logs and create a policy:

```sh
# Check for denials
sudo ausearch -m avc -ts recent

# Generate and install policy if needed
sudo ausearch -m avc -ts today | audit2allow -M check_vpn_local
sudo semodule -i check_vpn_local.pp
```

### Debian / Ubuntu

Use `cargo-deb` for easy .deb package creation:

```sh
cargo install cargo-deb
cargo deb
# Installs to /usr/bin/check_vpn with systemd unit
```

---

## Troubleshooting

**Service won't start:**
```sh
sudo journalctl -u check_vpn -b
# Check for config errors or permission issues
```

**Can't detect ISP:**
```sh
check_vpn --run-once --dry-run -vvv
# Verbose output shows API responses and ISP detection
```

**Permission denied errors:**
```sh
# Ensure service user has access to required directories
sudo chown -R checkvpn:checkvpn /etc/check_vpn /var/log/check_vpn
sudo chmod 755 /etc/check_vpn /var/log/check_vpn
```

**VPN not reconnecting:**
- Verify the action type and argument are correct
- Test the action manually (e.g., `sudo systemctl restart your-vpn-unit`)
- Check if the service user has permissions to execute the action
- For `restart-unit`, ensure the service user has appropriate PolicyKit rules or run as root

---

## Development

### Building from Source

Requirements:
- Rust 1.70 or later
- Cargo (comes with Rust)

```sh
git clone https://github.com/macg4dave/check_vpn_rust.git
cd check_vpn_rust
cargo build --release
```

The compiled binary will be at `target/release/check_vpn`.

### Running Tests

Run standard unit tests:

```sh
cargo test
```

Run integration tests (requires network access):

```sh
cargo test -- --ignored
```

Run all tests with verbose output:

```sh
cargo test -- --nocapture
```

### Project Structure

```
src/
├── main.rs           # Entry point
├── app.rs            # Main application logic
├── app_check.rs      # VPN check implementation
├── timer.rs          # Interval timing
├── logging.rs        # Logging setup
├── actions/          # Action execution (reboot, restart-unit, command)
├── cli/              # Command-line parsing
├── config/           # Configuration loading and validation
├── ip_api/           # IP lookup API client
├── metrics/          # HTTP metrics server
└── networking/       # Connectivity checks
```

### Building Packages

**Debian/Ubuntu .deb:**
```sh
cargo install cargo-deb
cargo deb
# Output: target/debian/check_vpn_*.deb
```

**Fedora/RHEL .rpm:**
```sh
# See contrib/check_vpn.spec for RPM spec file
rpmbuild -ba contrib/check_vpn.spec
```

**Docker:**
```sh
docker build -f contrib/Dockerfile -t check_vpn:latest .
```

### Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Run `cargo test` and `cargo clippy`
5. Commit your changes (`git commit -am 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Scripts

- `scripts/install-hooks.sh` - Install git pre-commit hooks
- `scripts/pre-commit.sh` - Run tests and linting before commits
- `contrib/run_acceptance.sh` - Run acceptance tests

---

## License

See [LICENSE](LICENSE) file for details.

## Support

- **Issues:** [GitHub Issues](https://github.com/macg4dave/check_vpn_rust/issues)
- **Discussions:** [GitHub Discussions](https://github.com/macg4dave/check_vpn_rust/discussions)

