# check_vpn (Rust)

[![CI](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/macg4dave/check_vpn_rust/actions/workflows/ci.yml)

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

Testing
-------

Some tests in this repository exercise real network endpoints and are marked ignored by default to avoid flakiness in CI or when running offline. Use the following commands to run tests that require network access.

Run all tests (mocked and unit tests):

```powershell
cargo test
```

Run only ignored tests (these include real-network integration tests):

```powershell
cargo test -- --ignored
```

Run a single ignored test (example):

```powershell
cargo test --test networking_testsreal real_ip_api_returns_200_and_parses -- --ignored
```

Notes:
- Mocked HTTP tests (using `httpmock`) cover common ip-api behaviors: 200, 500, 429, timeouts, and malformed responses. These run by default.
- Real-network integration tests are kept under `tests/*real.rs` and are ignored by default. Enable them when you have network access and want end-to-end verification.
- If you want to run only a subset of tests, use `cargo test <pattern>` or `cargo test --test <testfile>` as usual.

Testing in containers and CI
---------------------------

You can run the test suite inside a container (useful for reproducing CI environments) or configure CI workflows to run both unit/mocked tests and, optionally, real-network integration tests.

Example Dockerfile (run tests inside a Rust container):

```dockerfile
FROM rust:1.73-slim
WORKDIR /usr/src/check_vpn
COPY . .

# Install build tools for dependencies if needed (deb-based image)
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Run tests (default tests only)
RUN cargo test --verbose

# To run ignored tests (real network), execute at runtime with -- --ignored
# docker build -t check_vpn_tests .
# docker run --rm check_vpn_tests
# or run the ignored tests via an interactive container:
# docker run --rm -it check_vpn_tests bash -c "cargo test -- --ignored"
```

GitHub Actions example (run default tests on push, and a nightly job for ignored tests):

Create `.github/workflows/ci.yml` with:

```yaml
name: CI

on:
	push:
		branches: [ main ]
	pull_request:
		branches: [ main ]
	schedule:
		- cron: '0 3 * * *' # nightly run for optional integration tests (UTC)

jobs:
	test:
		runs-on: ubuntu-latest
		steps:
			- uses: actions/checkout@v4
			- name: Install Rust
				uses: dtolnay/gh-actions-rs@v1
				with:
					profile: minimal
					toolchain: stable
			- name: Cache cargo
				uses: actions/cache@v4
				with:
					path: |
						~/.cargo/registry
						~/.cargo/git
						target
					key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
			- name: Run tests (unit & mocked)
				run: cargo test --verbose

	# Optional job for running ignored real-network tests (scheduled or manual)
	real_tests:
		if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
		runs-on: ubuntu-latest
		steps:
			- uses: actions/checkout@v4
			- name: Install Rust
				uses: dtolnay/gh-actions-rs@v1
				with:
					profile: minimal
					toolchain: stable
			- name: Cache cargo
				uses: actions/cache@v4
				with:
					path: |
						~/.cargo/registry
						~/.cargo/git
						target
					key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-real
			- name: Run ignored real-network tests
				# These tests are marked #[ignore]; run them explicitly
				run: cargo test -- --ignored --nocapture
```

Notes and safety
- Running ignored real-network tests in CI can cause external network traffic and may occasionally fail due to remote service changes. Keep them in a separate scheduled or manually-triggered job.
- If tests require environment variables or secrets (e.g., private endpoints), add them as GitHub Secrets and pass them to the workflow using `env:` on the step.
- Use `-- --ignored` to execute ignored tests; use `cargo test <pattern>` to run a single test or file.

If you want, I can add the GitHub Actions workflow file to the repository and/or a Dockerfile under `contrib/` so you can run the containerized tests easily.

## Metrics endpoint & testing seam

Metrics endpoint
- When enabled via configuration or CLI the program starts a small HTTP server bound to the configured address. It exposes two simple endpoints useful for monitoring and health checks:
	- `/health` — returns HTTP 200 when the process is running; suitable for container liveness/readiness probes.
	- `/metrics` — returns a small plain-text metrics payload intended for scraping by Prometheus or simple monitoring tools (contains basic runtime/check information).

Example (when metrics are configured to bind to `127.0.0.1:9090`):

- Health: `http://127.0.0.1:9090/health`
- Metrics: `http://127.0.0.1:9090/metrics`

Testing seam: `perform_check`
- The single-iteration check logic is implemented in a test-friendly function re-exported as `check_vpn::app::perform_check`.
- `perform_check` is designed for dependency injection: in tests you can pass closures or mocks for the ISP lookup and the action runner so tests avoid real network calls and external side effects.

Example unit-test sketch:

```rust
let mut action_ran = false;
let cfg = /* build a minimal Config for the check */;
let get_isp = || -> Result<String, _> { Ok("SomeISP".to_string()) };
let run_action = |_: &str| -> Result<(), _> { action_ran = true; Ok(()) };

// Call the test seam. The exact signature is dependency-injected; this sketch shows the intent.
check_vpn::app::perform_check(&cfg, &get_isp, &run_action).unwrap();
assert!(action_ran);
```

Notes
- Prefer unit tests that inject mocks for `get_isp` and `run_action` (fast and deterministic).
- Real-network integration tests are available under `tests/*real.rs` and are ignored by default; run them explicitly with `cargo test -- --ignored` when needed.

Refactor notes (recent)
-----------------------

- The CLI argument handling has been reorganized into `src/cli/mod.rs` with the same public type `crate::cli::Args` kept for compatibility. This makes it easier to extend the CLI module with helpers and submodules.
- The negative connectivity test that previously used a remote/unroutable address has been replaced with a deterministic local simulation against a high loopback port (65000) to avoid flaky CI behavior.
- A small cleanup removed an intermediate legacy file and tidied module layout.
 - The `networking` code has been reorganized into a module folder for clarity: `src/networking/mod.rs` now contains the public API (`is_online`, `is_online_with_retries`, constants, etc.), with helper modules alongside it at `src/networking/connect.rs` (connection logic) and `src/networking/error.rs` (error types). Callers continue to use `crate::networking::...` with no API changes.

How to run tests (recap)
-----------------------

- Run all default tests (fast, mocked, unit):

```sh
cargo test
```

- Run ignored tests (real-network/integration tests) explicitly:

```sh
cargo test -- --ignored
```

If you'd like, I can add a small `contrib/` or CI job that runs the ignored integration tests on a schedule.