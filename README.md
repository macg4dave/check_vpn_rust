# check_vpn (Rust)

This repository contains a small Rust utility that checks if a VPN appears to be lost by checking the public ISP reported by ip-api.com. If the ISP matches a configured value (the ISP used when VPN is lost), the program runs an action such as rebooting or reconnecting the VPN.

This is a port of an existing `check_vpn.sh` script to Rust so you get a single compiled binary and better control over logging and flags.

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
- `--config-file`: Overrides config.xml path


Features
VPN Status Check: Determines if the VPN is active by checking the external ISP.
Automatic Action on VPN Loss: Executes a specified command when the VPN is disconnected, e.g., restart the VPN service, run a script or reboot.
Internet Connectivity Check: Only checks VPN status if there is internet access.

log_file="/var/log/check_vpn/check_vpn.log" # Log file path log_verbose=1 # Verbosity level: 1=ERROR, 2=WARN, 3=INFO, 4=DEBUG