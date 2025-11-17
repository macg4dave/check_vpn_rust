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

// Core modules using standard Rust module system
pub mod cli;
pub mod ip_api;
pub mod json_io;
pub mod metrics;
pub mod networking;
pub mod xml_io;

// Application logic
pub mod app;

// Shared helpers and modules
pub mod actions;
pub mod config;
pub mod fs_ops;
pub mod logging;

// Timer module (single file, included for consistency)
pub mod timer {
    include!("timer.rs");
}

// Convenience re-exports for common items used throughout tests and the
// examples in README. Keeping these re-exports stable makes the top-level
// crate more ergonomic to use.
pub use actions::parse_action;
pub use actions::Action;
pub use config::Config;
pub use ip_api::get_isp;
pub use networking::NetworkingError;
pub use timer::{start_timer, TimerHandle};
