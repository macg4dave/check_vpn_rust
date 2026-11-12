#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
//! check_vpn - small CLI tool and library to verify external IP / ISP and
//! run recovery actions when the public ISP matches the configured normal ISP.
//!
//! This crate exposes both a binary (`check_vpn` in `src/main.rs`) and a
//! library surface for testing and embedding. The public API is intentionally
//! small and centered around:
//! - `check_vpn::app::perform_check` - perform a single provider-first ISP check
//! - `check_vpn::config::Config` - load/merge/validate configuration
//! - `check_vpn::actions` - action parsing and execution helpers
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

pub mod actions;
pub mod app;
pub mod cli;
pub mod completions;
pub mod config;
pub mod error_types;
pub mod fs_ops;
pub mod init_wizard;
pub mod ip_api;
pub mod logging;
pub mod networking;
pub mod providers;
pub mod xml_io;

/// Run the application given a pre-parsed set of CLI arguments. Mirrors the
/// behavior of the binary entry point and returns `Result<(), AppError>` so
/// callers can inspect and handle failures programmatically (e.g., tests).
pub fn try_run_with_args(
    args: crate::cli::Args,
) -> std::result::Result<(), crate::error_types::AppError> {
    use crate::error_types::AppError;
    use anyhow::Context;

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

    let source = crate::config::ConfigSource::resolve(args.config.as_deref())
        .map_err(AppError::ConfigLoadError)?;
    let cfg = source
        .load()
        .context("loading configuration")
        .map_err(AppError::ConfigLoadError)?;
    crate::app::run_with_source(args, cfg, source)
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
