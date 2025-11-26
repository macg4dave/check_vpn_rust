# Project — Copilot / AI Agent Instructions (Template)
## Enter project info here
Purpose: Provide concise, project-wide context and expectations for an AI coding agent working in this Rust repository.

## Project Overview

* Rust application: `<brief project summary here>`
* Architecture: `<CLI / TUI / web service / library / whatever>`
* Goals: `<performance / correctness / portability / simplicity>`

## Folder Structure (fill in as appropriate)

* `/src/` — main source code
* `/tests/` — integration tests
* `/crates/<name>/` — workspace members (if any)
* `/docs/` — documentation
# check_vpn — Copilot / AI Agent Instructions

Purpose: give an AI coding agent the minimal, practical knowledge to be productive in this repository.

**Big Picture**
- **What it is:** single binary Rust daemon that verifies the public ISP/IP (via `ip-api.com`) and triggers configured actions when the VPN/ISP changes.
- **Primary flow:** `main` -> `try_main()` (thin) -> CLI parsing (`src/cli`) -> config load/merge (`src/config`) -> `app::run` / `app::perform_check` -> call `ip_api::get_isp` -> evaluate and use `actions` + `actions::runner` to execute.
- **Synchronous design:** code uses blocking I/O (e.g., `reqwest::blocking`, `zbus::blocking`) and prefers D-Bus for system actions where possible.

**Key files / entry points**
- `src/main.rs` — thin entry; keep `try_main()` behavior unchanged when modifying startup.
- `src/lib.rs` — crate exports and module map (useful for tests and embedding).
- `src/app/check.rs` and `src/app/mod.rs` — main app logic and single-check implementation (`perform_check`).
- `src/ip_api/client.rs` — HTTP client and retry behavior (env vars: `CHECK_VPN_TEST_URL`, `CHECK_VPN_RETRY_COUNT`, `CHECK_VPN_MAX_RESPONSE_BYTES`).
- `src/actions/runner.rs` — production `ActionRunner` that prefers D-Bus for `reboot` and `restart-unit`, fallback to shell for `command`.
- `src/logging.rs` — logging init; CLI verbosity is applied early in `try_main()`.
- `src/timer.rs` — simple recurring timer utilities used by the daemon.

**Project-specific conventions & patterns**
- Keep `main` minimal; new behavior should be added inside `try_main()` or `app` functions so tests can exercise logic without calling `std::process::exit`.
- Prefer injecting dependencies for testability. Many functions take clients/closures (see `ip_api::get_isp_with_client_and_url`).
- Use the provided `ActionRunner` trait to implement or mock action execution in tests; real runner uses system D-Bus via `zbus` and must not be invoked in non-root CI.
- Tests often use env overrides (see `CHECK_VPN_TEST_URL`, `CHECK_VPN_MAX_RESPONSE_BYTES`). Honor those when adding integration/unit tests.
- Respect `dry_run` flags in code paths that have side effects — tests commonly pass `dry_run=true` to avoid system changes.

**Build / test / debug workflow (practical commands)**
- Build release binary: `cargo build --release` (binary: `target/release/check_vpn`).
- Run unit + integration tests: `cargo test`.
- Run ignored integration tests: `cargo test -- --ignored` (used for longer or network tests).
- Run binary locally with config and verbose logs: `./target/debug/check_vpn --config /path/to/config.xml --dry-run --log-verbose 3`.

**Packaging & installer helpers**
- Scripts for packaging and install live under `contrib/` and `scripts/` (e.g., `contrib/fpm-build.sh`, `scripts/install-windows.ps1`, `scripts/install.sh`). Use these for distro packaging or to reproduce CI packaging behavior.

**Testing notes & CI**
- Many tests are integration-style and rely on determinism via env vars and test servers (see `tests/` and `httpmock` use in `src/ip_api` tests).
- Avoid touching system D-Bus or issuing reboots in CI — use `dry_run` or the `ActionRunner` trait to inject a test runner.

**When changing behavior**
- Update/extend tests in `tests/` and local unit tests under `src/` as appropriate; CI requires `cargo test` to pass.
- Preserve top-level CLI behavior and configuration precedence: explicit `--config` path overrides on-disk defaults.
- If adding external dependencies (native libs or system APIs), document platform-specific steps in `DEVELOPING.md` and `README.md`.

**Small examples**
- To mock IP API endpoint in tests: set `CHECK_VPN_TEST_URL` to a local mock server URL.
- To test large-response handling, set `CHECK_VPN_MAX_RESPONSE_BYTES` in test scope.

If anything here is unclear or you want a different emphasis (e.g., more architecture diagrams, CI workflow details, or examples of safe test runners), tell me which areas to expand and I will iterate.
* Add or update tests for any altered behavior.
