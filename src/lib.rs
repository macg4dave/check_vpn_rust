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
pub mod networking;
pub mod providers;
pub mod init_wizard;
pub mod completions;

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

// Shared helpers and modules
pub mod fs_ops;
pub mod actions;
pub mod logging;
pub mod config;
pub mod timer {
	include!("timer.rs");
}

// Convenience re-exports for common items used throughout tests and the
// examples in README. Keeping these re-exports stable makes the top-level
// crate more ergonomic to use.
pub use actions::Action;
pub use actions::parse_action;
pub use ip_api::get_isp;
pub use providers::{VpnIdentity};
pub use timer::{start_timer, TimerHandle};
pub use networking::NetworkingError;
pub use config::Config;
