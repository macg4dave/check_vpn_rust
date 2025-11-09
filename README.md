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
 - `--connectivity-timeout <secs>`: timeout for connectivity checks (default 2)
 - `--connectivity-retries <n>`: number of attempts for connectivity checks before declaring offline (default 1)


Features
VPN Status Check: Determines if the VPN is active by checking the external ISP.
Automatic Action on VPN Loss: Executes a specified command when the VPN is disconnected, e.g., restart the VPN service, run a script or reboot.
Internet Connectivity Check: Only checks VPN status if there is internet access.

Configuration
The program reads configuration from XML (see `examples/check_vpn.xml`) or from command-line flags. A new optional field `connectivity_retries` controls how many times the program will retry connectivity probes before concluding the internet is down. This can also be overridden with the CLI flag `--connectivity-retries`.

Exit codes
- `2` — configuration validation failed on startup. The program prints each validation error as a separate log line before exiting.
 - `3` — DNS/name resolution failure occurred while attempting connectivity checks.
 - `4` — Generic connectivity failure (unreachable/timeout) when considered fatal.
 - `5` — Failed to determine ISP from the IP API when considered fatal.

By default the program only exits with non-zero codes for configuration validation errors. When invoking with `--run-once` it will also exit with the connectivity/ISP exit codes described above. Use `--exit-on-error` to force the process to exit with the same codes even in long-running mode (useful for container health checks or external monitors).

log_file="/var/log/check_vpn/check_vpn.log" # Log file path log_verbose=1 # Verbosity level: 1=ERROR, 2=WARN, 3=INFO, 4=DEBUG