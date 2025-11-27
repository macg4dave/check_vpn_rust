INSTALL
=======

This repository includes a small installer script for the `check_vpn` binary at `scripts/install.sh`.
The installer supports interactive and non-interactive installs, per-user or system installs, Linux (systemd) and macOS (launchd), and a `--dry-run` mode for safe verification.

This document shows how to use the installer, the available flags, CI-friendly examples, and how to run the included tests.

Requirements
------------
- A built `check_vpn` binary (default path: `target/release/check_vpn`). Build with:

```bash
cargo build --release
```

- On Linux: `systemctl` is used for systemd operations (system or user). On macOS: `launchctl` is used for launchd operations.

Installer overview
------------------
Location: `scripts/install.sh`

Key features:
- Interactive prompts for install type (system vs user) if `--mode` isn't provided.
- Non-interactive flags for use in scripts and CI (`--mode`, `--yes`, `--dry-run`, `--no-start`).
- Fallback service/plist files are checked in `contrib/` when not provided explicitly.
- `--dry-run` prints planned actions instead of making changes. Use this to verify what will happen.

Usage
-----
Basic interactive install (recommended for local setups):

```bash
# system install (requires sudo)
sudo ./scripts/install.sh

# per-user install (no sudo)
./scripts/install.sh
```

Non-interactive (useful for CI or automation):

```bash
# System install on Linux (requires sudo):
sudo ./scripts/install.sh --mode system --yes

# Per-user non-interactive install:
./scripts/install.sh --mode user --yes

# Dry-run: show what would be done without making any changes
./scripts/install.sh --mode user --dry-run

# Install but do not start/enable the service
sudo ./scripts/install.sh --mode system --no-start --yes
```

Flags
-----
- `--binary <path>`: Path to the built `check_vpn` binary. Defaults to `target/release/check_vpn`.
- `--config <path>`: Path to a config XML file to install. Defaults to `./check_vpn.xml`.
- `--service <path>`: Path to a systemd service file (Linux). If not provided the installer will look in `contrib/check_vpn.service`.
- `--mode <interactive|system|user|auto>`: Picks installation mode. `interactive` prompts the user; `system` installs system-wide; `user` installs into user locations.
- `--yes`: Assume "yes" for interactive confirmations (CI-friendly).
- `--no-start`: Install files but don't attempt to start/enable the service/unit.
- `--dry-run`: Print the actions that would be taken instead of performing them.
- `-h, --help`: Show help and exit.

Platform specifics
------------------
Linux
- System install: binary -> `/usr/local/bin/check_vpn`, config -> `/etc/check_vpn/config.xml`, systemd unit -> `/etc/systemd/system/check_vpn.service`.
- User install: binary -> `${XDG_BIN_HOME:-$HOME/.local/bin}/check_vpn` (or `~/bin` if on PATH), config -> `${XDG_CONFIG_HOME:-$HOME/.config}/check_vpn/config.xml`, user systemd unit -> `~/.config/systemd/user/check_vpn.service`.

macOS
- System install: binary -> `/usr/local/bin/check_vpn`, config -> `/etc/check_vpn/config.xml`, plist -> `/Library/LaunchDaemons/com.check_vpn.plist`.
- User install: binary -> `~/.local/bin` or `~/bin`, config -> `~/.config/check_vpn/config.xml`, plist -> `~/Library/LaunchAgents/com.check_vpn.plist`.

CI examples
-----------
Here are example snippets you can paste into CI job steps. They run the installer in non-interactive dry-run or real install modes.

GitHub Actions (dry-run check):

```yaml
- name: Build
  run: cargo build --release

- name: Installer dry-run
  run: ./scripts/install.sh --mode user --binary target/release/check_vpn --config ./check_vpn.xml --dry-run --yes
```

GitHub Actions (attempt user install into workspace local dir - safe for ephemeral runners):

```yaml
- name: Install (user, no services)
  run: ./scripts/install.sh --mode user --binary target/release/check_vpn --config ./check_vpn.xml --yes --no-start
```

Notes for CI
- Use `--dry-run` to verify actions without changing the runner.
- Avoid system install in CI runners unless you explicitly want to modify the environment; prefer per-user install or `--no-start` to avoid altering services.

Testing the installer
---------------------
There are integration tests that exercise the installer in dry-run mode and verify expected output. Run them with:

```bash
cargo test --test installer_tests -- --nocapture
```

Troubleshooting
---------------
- If the installer complains about missing binary, build the release binary first:

```bash
cargo build --release
```

- If a system install is requested but you see a permissions error, re-run with `sudo`.
- If you rely on the `contrib` service/plist files, ensure they exist at `contrib/check_vpn.service` or `contrib/check_vpn.plist`.

Security and safety notes
-------------------------
- The script attempts to avoid surprises: files are copied with safe permissions and the `--dry-run` option is available for verification.
- System installs modify privileged locations and start services; only run them on systems you control.

Support & next steps
--------------------
If you'd like, I can add:
- A `--prefix` to control install root (for cross-platform packaging).
- A `--summary` dry-run output that prints only the file changes planned.
- More tests to cover system-mode errors (e.g., expected failure when not run as root).

Thanks — if you'd like a specific CI snippet for a runner (GitLab, CircleCI, etc.), tell me which one and I will add it.
