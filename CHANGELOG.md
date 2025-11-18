# Changelog

## Unreleased

- Add config validation with detailed, user-friendly error messages. Validation now reports multiple problems at once and the service exits with a specific exit code on startup when configuration is invalid.
- Add `connectivity_retries` configuration (defaults to 1) and CLI option `--connectivity-retries` to control how many times connectivity checks are retried.
- Wire retries into connectivity checks so transient failures are tolerated when configured.
- Update tests and docs to reflect new validation and retry behavior.
 - Add explicit exit codes for connectivity and ISP failures (DNS failure, connectivity failure, ISP lookup failure).
 - Add CLI flag `--exit-on-error` to force non-zero exit codes on errors even in long-running mode (useful for health checks).
 - Re-export `NetworkingError` from crate root for easier error handling by embedders.
 - Refactor CLI module into a directory-style module at `src/cli/mod.rs`.
	 - Public API preserved (`crate::cli::Args`).
	 - Improved docs and inline unit tests for argument parsing.
 - Tests: make negative connectivity case deterministic by testing a high-numbered loopback port instead of relying on external networks.
 - Remove legacy `src/cli.rs` file and tidy module layout.
 - Minor portability fixes: wrap environment variable mutation calls in `unsafe` blocks where required by some targets.

- Add CLI flag `-c, --config <FILE>` to allow specifying a custom XML config path on the command line. When provided this path takes precedence over the `CHECK_VPN_CONFIG` env var and the default lookup order. (PR: local)
