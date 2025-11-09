# check_vpn (Rust)

This repository contains a small Rust utility that checks if a VPN appears to be lost by checking the public ISP reported by ip-api.com. If the ISP matches a configured value (the ISP used when VPN is lost), the program runs an action such as rebooting or reconnecting the VPN.

This is a port of an existing `check_vpn.sh` script to Rust so you get a single compiled binary and better control over logging and flags.

Important assumptions
- The program expects to run on a Unix-like system (Linux). The `ping` invocation uses `-c` and `-W` options on non-Windows systems.
- Running a reboot/shutdown action requires root privileges.

Build

You need Rust and cargo installed. Then:

```sh
cargo build --release
```

The resulting binary will be at `target/release/check_vpn`.

Run (dry-run)

```sh
target/release/check_vpn --dry-run
```

Common options
- `--interval <seconds>`: seconds between checks (default 60)
- `--isp-to-check`: ISP string that indicates VPN is lost (default set to the original script value)
- `--vpn-lost-action`: shell command to run when VPN is lost (default `/sbin/shutdown -r now`)
- `--dry-run`: log the action instead of executing it

Installing as a systemd service

You can adapt the existing `check_vpn.service` and point `ExecStart` to the compiled binary (for example `/usr/local/bin/check_vpn/check_vpn`). Example `ExecStart`:

```
ExecStart=/usr/local/bin/check_vpn/check_vpn --interval 60
```

Then install unit and enable it:

```sh
sudo cp target/release/check_vpn /usr/local/bin/check_vpn/check_vpn
sudo cp check_vpn.service /etc/systemd/system/check_vpn.service
sudo systemctl daemon-reload
sudo systemctl enable --now check_vpn.service
```

Notes & next steps
- I implemented a conservative, minimal Rust port. Next iteration could add retries/backoff, better detection (interface checks), or an option to call an external script such as an OpenVPN management command to reconnect.
