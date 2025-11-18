# check_vpn — A Tiny, Powerful VPN Watchdog

A compact Rust-powered tool that does one job exceptionally well: **instantly detect when your VPN drops and take action automatically.**  
A lightweight binary you can trust on servers, desktops, and homelabs.

If you rely on a VPN for privacy, automation, or secure routing, this tool keeps an eye on your connection 24/7 and reacts the moment your VPN slips with near‑zero overhead.

## Why you want this

- **Ridiculously easy to use.** One binary, one config file, one installer.  
- **Small but potent** — lightweight Rust binary, fast startup, tiny footprint.  
- **Lightning fast.** Written in Rust, runs with practically zero overhead.  
- **Instant detection** — checks your public-facing ISP and reacts the moment it changes. 
- **Flexible responses** — reboot the machine, restart a systemd unit, or run any script/command you want.  
- **Perfect for homelabs, seedboxes, servers, and desktops.**

# check_vpn — The No‑Nonsense VPN Watchdog  
A tiny Rust daemon that does one thing *perfectly*: **instantly detect when your VPN dies and take action before anything leaks.**

It’s brutally simple, rock‑solid, and built for people who actually rely on their VPN — homelabbers, seedboxes, small servers, power users, and anyone who hates downtime and surprises.

Forget bloated “network monitors.” This is a **single binary**, near‑zero overhead, and absolutely ruthless at catching VPN dropouts.

---

[![CI](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/macg4dave/check_vpn_rust/actions)
[![Releases](https://img.shields.io/github/v/release/macg4dave/check_vpn_rust)](https://github.com/macg4dave/check_vpn_rust/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Supported platforms & requirements

- Supported: Linux (systemd) and macOS (launchd).
- Rust: Rust stable is required; see `DEVELOPING.md` for exact toolchain notes and optional build dependencies.
- macOS build notes: you may need `openssl`, `pkg-config`, and `curl` installed (see Developer Notes below).

Table of contents

- Quick Start
- Configuration (default locations and examples)
- CLI examples
- Running as a service (systemd / launchd)
- Troubleshooting
- Developer notes
- Contributing
- Security & Privacy
- License

---

## Quick Start (in under 30 seconds)

### 1. Build or download
```bash
cargo build --release
# Output: target/release/check_vpn
```
Or grab a prebuilt binary from Releases.

### 2. Test without triggering actions  
```bash
./check_vpn --dry-run
```

### 3. Install (zero thinking required)

**System-wide (recommended for servers):**
```bash
sudo ./scripts/install.sh
```

**User-only (no root needed):**
```bash
./scripts/install.sh
```

Installer handles:
- copying the binary  
- placing your config  
- installing + enabling service (systemd or launchd)

---

## Minimal XML Configuration

Place it at either:
- `/etc/check_vpn/config.xml` (system)
- `$HOME/.config/check_vpn/config.xml` (user)

```xml
<?xml version="1.0" encoding="utf-8"?>
<config>
  <!-- How often to check, in seconds -->
  <interval>60</interval>

  <!-- Your “non‑VPN” ISP name from ip-api.com -->
  <isp_to_check>Your Public ISP Here</isp_to_check>

  <!-- Action when VPN is lost: reboot | restart-unit | command -->
  <vpn_lost_action_type>reboot</vpn_lost_action_type>

  <!-- Argument for that action -->
  <vpn_lost_action_arg>/sbin/shutdown -r now</vpn_lost_action_arg>

  <!-- For testing without executing actions -->
  <dry_run>false</dry_run>
</config>
```

Actions you can trigger:
- **reboot** (full system restart)  
- **restart-unit** (systemd service)  
- **command** (any shell command or script)

---

## Handy Command Examples

Verbose dry-run:
```bash
./check_vpn --config /etc/check_vpn/config.xml --dry-run --log-verbose 3
```

Run normally (continuous watchdog):
```bash
./check_vpn
```

---

## Installer Flags (for automation / CI)

- `--binary <path>`
- `--config <path>`
- `--service <path>`
- `--mode <interactive|system|user|auto>`
- `--yes` (non-interactive)
- `--no-start`
- `--dry-run`

Ideal for scripted installs or cluster deployments.

---

## Running as a service (systemd / launchd)

Below are minimal examples showing how to run `check_vpn` as a system service. The repository includes example service files in `contrib/`:

- `contrib/check_vpn.service` (systemd)
- `contrib/check_vpn.plist` (launchd)

Systemd (system-wide) example — place as `/etc/systemd/system/check_vpn.service`:

```ini
[Unit]
Description=check_vpn watchdog
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/check_vpn --config /etc/check_vpn/config.xml
Restart=on-failure
RestartSec=10
User=root
SyslogIdentifier=check_vpn

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
```

If you prefer a per-user systemd unit, set `ExecStart` to point at the user config (`$HOME/.config/check_vpn/config.xml`) and enable it in the user systemd scope.

Launchd (macOS) example — place as `/Library/LaunchDaemons/com.macg4dave.check_vpn.plist` and ensure root ownership:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.macg4dave.check_vpn</string>
    <key>ProgramArguments</key>
    <array>
      <string>/usr/local/bin/check_vpn</string>
      <string>--config</string>
      <string>/Library/Application Support/check_vpn/config.xml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/var/log/check_vpn.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/check_vpn.err</string>
  </dict>
</plist>
```

Load the plist (macOS):

```bash
sudo chown root:wheel /Library/LaunchDaemons/com.macg4dave.check_vpn.plist
sudo launchctl load /Library/LaunchDaemons/com.macg4dave.check_vpn.plist
```

Notes:
- The examples above assume the binary is installed to `/usr/local/bin` and the system config is under `/etc/check_vpn` or `/Library/Application Support/check_vpn`. Adjust paths to match your installation (the install scripts in `scripts/` and `contrib/` may already do this).
- Actions that reboot the machine or restart system services require appropriate privileges; run the service as root or use the user-level configuration where appropriate.

---

## Troubleshooting

### Install the service (quick snippets)

If you prefer the repo installer, it accepts flags for automation. Example (system mode):

```bash
# from the repository root
sudo ./scripts/install.sh --binary /usr/local/bin/check_vpn \
  --config /etc/check_vpn/config.xml \
  --service /etc/systemd/system/check_vpn.service \
  --mode system --yes
```

Manual systemd install (if you want to copy files yourself):

```bash
sudo install -Dm755 target/release/check_vpn /usr/local/bin/check_vpn
sudo mkdir -p /etc/check_vpn
sudo cp contrib/check_vpn.service /etc/systemd/system/check_vpn.service
sudo cp /path/to/your/config.xml /etc/check_vpn/config.xml
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
```

Manual launchd install (macOS):

```bash
sudo install -Dm755 target/release/check_vpn /usr/local/bin/check_vpn
sudo mkdir -p "/Library/Application Support/check_vpn"
sudo cp /path/to/your/config.xml "/Library/Application Support/check_vpn/config.xml"
sudo cp contrib/check_vpn.plist /Library/LaunchDaemons/com.macg4dave.check_vpn.plist
sudo chown root:wheel /Library/LaunchDaemons/com.macg4dave.check_vpn.plist
sudo launchctl load /Library/LaunchDaemons/com.macg4dave.check_vpn.plist
```

### Verify the service is running

Systemd:

```bash
sudo systemctl status check_vpn.service
sudo journalctl -u check_vpn.service --no-pager -n 200
```

Launchd (macOS):

```bash
sudo launchctl list | grep com.macg4dave.check_vpn
sudo launchctl list com.macg4dave.check_vpn
# Show recent logs (macOS unified logging):
sudo log show --predicate 'process == "check_vpn"' --last 1h
```


**Binary missing?**
```bash
cargo build --release
```

**Systemd service failing?**
```bash
sudo journalctl -u check_vpn.service --no-pager
sudo systemctl status check_vpn.service
```

**macOS launchd quirks:**  
System plists must be root-owned and placed in `/Library/LaunchDaemons/`.

---

## Developer Notes

- Rust stable required  
- macOS users may need:
```bash
brew install openssl pkg-config curl
export OPENSSL_DIR="$(brew --prefix openssl)"
```

**Run tests:**
```bash
cargo test
```

**Run ignored integration tests:**
```bash
cargo test -- --ignored
```

Docker-based reproducible builds are included for Linux packaging.

---

## Packaging & Releases

The repository includes produced package artifacts under `artifacts/` and helper scripts in `contrib/` and `scripts/` to build distribution packages (Deb, RPM, macOS package/plist, and installer wrappers).

Where to look

- Prebuilt outputs: `artifacts/` (contains `debian/`, `fedora/`, `macos/` subfolders).
- Packaging helpers: `contrib/` (e.g. `contrib/cargo-deb.sh`, `contrib/fpm-build.sh`) and `scripts/` (Docker-based build helpers such as `scripts/docker-build.sh`).

Quick example (build binary + package locally)

```bash
# 1) Build the release binary
cargo build --release

# 2) Run one of the provided packaging helpers (examples live in contrib/)
#    Adjust the script name/flags as needed; scripts may package the built binary
./contrib/fpm-build.sh    # builds RPM/DEB via fpm (script-specific flags may apply)
./contrib/cargo-deb.sh    # helper to produce a .deb using cargo-deb
```

Build for different systems

- Linux (x86_64): build on a Linux host or use the Docker helper `scripts/docker-build.sh` to produce reproducible artifacts.
- Linux (musl/static): use a musl toolchain or cross compilation; consider `cross` or a Docker image that has a musl toolchain installed.
- macOS: build on macOS (required for signing/notarization) or use a macOS GitHub Actions runner; local macOS builds need Xcode toolchain and any native deps (see Developer Notes).
- Cross-compilation tips:
  - Use `cargo build --target <target-triple>` for Rust cross builds (install the target with rustup).
  - For packaging native OS packages (deb/rpm/pkg), it's usually easiest to run the packaging step on the target OS (Docker or CI runners are ideal).

CI / Reproducible builds

If you want automated packaging, the project contains Docker scripts and CI workflows (see `.github/workflows/` if present) — these can be adapted to produce `artifacts/` automatically on release.

If you'd like, I can add a short example GitHub Actions workflow that builds release artifacts for Linux and macOS and uploads them to Releases.

---

## Troubleshooting — common errors

1. Service won't start / permission denied

   - Symptom: `systemctl start` fails with `permission denied` or `exec format error`.
   - Fixes:
     - Ensure `ExecStart` path points to an executable built for the target architecture and is executable (`chmod +x /usr/local/bin/check_vpn`).
     - Check ownership and SELinux/AppArmor policies on the binary and config file.
     - For macOS, ensure the plist is owned by `root:wheel` and placed in `/Library/LaunchDaemons/`.

2. Config parse error / invalid XML

   - Symptom: `check_vpn` exits quickly with a config-related error.
   - Fixes:
     - Validate your XML (missing closing tags, invalid characters). Use `xmllint --noout /path/to/config.xml`.
     - Ensure the config file exists at the path given to `--config` or in the default location.

3. Actions not executed (reboot/restart-unit/command doesn't run)

   - Symptom: `check_vpn` reports detection but the action doesn't run.
   - Fixes:
     - Actions that restart services or reboot the machine require appropriate privileges. If running as a non-root user, use user-scoped units or grant the necessary privileges.
     - Verify `vpn_lost_action_arg` is correct (e.g., `systemctl restart your.service` for `restart-unit` or a full command for `command`).

4. Incorrect `isp_to_check` value / false positives

   - Symptom: `check_vpn` thinks VPN is down even when it isn't.
   - Fixes:
     - Confirm the `isp_to_check` value by running `curl -s https://ip-api.com/json | jq -r '.isp'` from the machine.
     - Be aware of ip-api rate limits; intermittent API failures can cause transient detection. Consider a slightly longer `interval` or a local probe endpoint for production.

5. Logs are empty / not helpful

   - Symptom: not enough information to diagnose a failure.
   - Fixes:
     - Run manually with verbosity for debugging: `./check_vpn --dry-run --log-verbose 3 --config /path/to/config.xml`.
     - Check system logs: `sudo journalctl -u check_vpn.service` (systemd) or `sudo log show --predicate 'process == "check_vpn"' --last 1h` (macOS).


## Contributing

PRs welcome. Keep the code clean, documented, and tested. The tool stays small, sharp, and focused.

---

## License

See `LICENSE` in the repository root.

