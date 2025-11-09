// The CLI module is defined inline here via `include!` to avoid ambiguity
// when both `src/cli.rs` and `src/cli/mod.rs` exist in the tree during a
// transitional refactor. The included file continues to provide
// `crate::cli::Args` and the public API expected by the rest of the crate.
pub mod cli;
pub mod networking;
pub mod ip_api;
pub mod metrics;
// The `app` module is implemented in `src/app.rs` and may contain submodules
// in `src/app/*.rs`. To avoid ambiguity between `src/app.rs` and
// `src/app/mod.rs` on some filesystems, include the file here explicitly.
pub mod app {
	include!("app.rs");
}
pub mod xml_io;
pub mod json_io;
// Canonical actions module (refactored into `src/actions`)
pub mod actions;
pub mod logging;
pub mod config;
pub mod timer;

// Re-export commonly used items for tests and external use
pub use actions::Action;
pub use actions::parse_action;
pub use ip_api::get_isp;
pub use timer::{start_timer, TimerHandle};
pub use networking::NetworkingError;
