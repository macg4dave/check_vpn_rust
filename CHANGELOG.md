# Changelog

## Unreleased

- No unreleased changes yet.

## v1.0.0

- Add config validation with detailed, user-friendly error messages. Validation now reports multiple problems at once and the service exits with a specific exit code on startup when configuration is invalid.
- Add `connectivity_retries` configuration (defaults to 1) and CLI option `--connectivity-retries` to control how many times connectivity checks are retried.
- Wire retries into connectivity checks so transient failures are tolerated when configured.
- Update tests and docs to reflect new validation and retry behavior.
- Add explicit exit codes for connectivity and ISP failures (DNS failure, connectivity failure, ISP lookup failure).
- Add CLI flag `--exit-on-error` to force non-zero exit codes on errors even in long-running mode (useful for health checks).
- Re-export `NetworkingError` from crate root for easier error handling by embedders.
- Refactor CLI module into a directory-style module at `src/cli/mod.rs` while keeping the public API (`crate::cli::Args`).
- Tests: make negative connectivity case deterministic by testing a high-numbered loopback port instead of relying on external networks.
- Remove legacy `src/cli.rs` file and tidy module layout.
- Add CLI flag `-c, --config <FILE>` to allow specifying a custom XML config path on the command line. When provided this path takes precedence over the `CHECK_VPN_CONFIG` env var and the default lookup order. (PR: local)
- Remove the built-in HTTP metrics/health server and associated CLI flags (`--enable-metrics`, `--metrics-addr`). Metrics are intentionally out-of-scope for this build; docs/tests were aligned.
- Fix long-interval sleep overflow in the main loop (sleep slices now use u64 without overflow).
- Improve provider chain diagnostics by logging provider construction failures.
- Gate systemd/D-Bus actions to Linux, add safer cross-platform fallbacks, and make non-Linux builds avoid systemd dependencies.
- Clean up documentation (ASCII headings, release process now semver starting at v1.0.0) and remove outdated cleanup summary.
