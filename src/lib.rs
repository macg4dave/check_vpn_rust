#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
//! check_vpn — small CLI tool and library to verify external IP / ISP and
//! optionally run actions when the public IP changes (useful for VPN checks).
//!
//! This crate exposes both a binary (`check_vpn` in `src/main.rs`) and a
//! library surface for testing and embedding. The public API is intentionally
//! small and centered around:
//! - `check_vpn::app::perform_check` — perform a single connectivity/ISP check
//! - `check_vpn::config::Config` — load/merge/validate configuration
//! - `check_vpn::timer::start_timer` and `TimerHandle` — a tiny recurring timer
//! - `check_vpn::actions` — action parsing and execution helpers
//!
//! Example (ignored):
//! ```ignore
//! // The primary workflow is exercised in the tests; this example is for
//! // documentation only and is not compiled as a doctest.
//! use check_vpn::{app::perform_check, ip_api::get_isp, actions::run_action};
//! // Construct an `EffectiveConfig` via merging CLI args and `Config` in real
//! // programs. For docs we only show the call site:
//! // perform_check(&effective_config, get_isp, run_action).unwrap();
//! ```

// Core modules. Some modules are inline-included to avoid duplicate-module
// compilation issues during an ongoing directory-style refactor — the
// include! usage is deliberate and safe. Prefer directory modules where the
// layout is stable.
pub mod cli;
pub mod completions;
pub mod init_wizard;
pub mod networking;
pub mod providers;

pub mod ip_api {
    include!("ip_api/mod.rs");
}

pub mod metrics;

pub mod app {
    include!("app.rs");
}

pub mod xml_io {
    include!("xml_io/mod.rs");
}

pub mod json_io {
    include!("json_io/mod.rs");
}

pub mod error_types;

// Shared helpers and modules
pub mod actions;
pub mod config;
pub mod fs_ops;
pub mod logging;
pub mod timer {
    include!("timer.rs");
}

// High-level entry helper designed for tests and embedding. This extracts the
// logic that `src/main.rs` previously kept internal so tests can invoke the
// same startup/arg handling path without depending on the binary's `main`.
/// Run the application given a pre-parsed set of CLI arguments. Mirrors the
/// behavior of the binary `try_main` and returns `Result<(), AppError>` so
/// callers can inspect and handle failures programmatically (e.g., tests).
pub fn try_run_with_args(args: crate::cli::Args) -> std::result::Result<(), crate::error_types::AppError> {
    // Keep most of the original `try_main` semantics here. This is intentionally
    // implemented at the crate root to keep the API surface small and testable.
    use anyhow::Context;
    use crate::error_types::AppError;

    // Initialize logging early using CLI verbosity so app.run can log at the
    // requested level. This respects RUST_LOG if already set.
    crate::logging::init_with_verbosity(args.verbose);

    // If wizard requested, run it and return early.
    if args.init {
        crate::init_wizard::run_init(args.init_output.as_deref(), args.init_no_fetch)
            .map_err(AppError::RuntimeError)?;
        return Ok(());
    }

    // Generate shell completions when requested and return early.
    if let Some(shell) = args.generate_completions.as_deref() {
        crate::completions::generate_completions(shell, args.completions_output.as_deref())
            .map_err(AppError::RuntimeError)?;
        return Ok(());
    }

    // Load configuration from the chosen source, wrapping errors as AppError.
    let cfg = if let Some(path) = args.config.as_ref() {
        match crate::config::Config::load_from_path(path.as_path()).context("loading configuration from --config path") {
            Ok(c) => c,
            Err(e) => {
                // Provide an actionable hint including a best-effort path
                // representation (safe for non-UTF8 paths).
                let cfg_path = args
                    .config
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .or_else(|| std::env::var("CHECK_VPN_CONFIG").ok())
                    .unwrap_or_else(|| {
                        "(no explicit config path set; looked up defaults)".to_string()
                    });

                log::error!(
                    "Failed to load config (path = {}): {:?}. If this is unintended, fix the XML or use --config to specify a different file.",
                    cfg_path,
                    e
                );
                return Err(AppError::ConfigLoadError(e));
            }
        }
    } else {
        match crate::config::Config::load().context("loading configuration via lookup") {
            Ok(c) => c,
            Err(e) => {
                let cfg_path = args
                    .config
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .or_else(|| std::env::var("CHECK_VPN_CONFIG").ok())
                    .unwrap_or_else(|| {
                        "(no explicit config path set; looked up defaults)".to_string()
                    });

                log::error!(
                    "Failed to load config (path = {}): {:?}. If this is unintended, fix the XML or use --config to specify a different file.",
                    cfg_path,
                    e
                );
                return Err(AppError::ConfigLoadError(e));
            }
        }
    };

    crate::app::run(args, cfg)?;
    Ok(())
}

// Convenience re-exports for common items used throughout tests and the
// examples in README. Keeping these re-exports stable makes the top-level
// crate more ergonomic to use.
pub use actions::parse_action;
pub use actions::Action;
pub use config::Config;
pub use ip_api::get_isp;
pub use networking::NetworkingError;
pub use providers::VpnIdentity;
pub use timer::{start_timer, TimerHandle};
