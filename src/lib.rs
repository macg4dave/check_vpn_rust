// The CLI module is defined inline here via `include!` to avoid ambiguity
// when both `src/cli.rs` and `src/cli/mod.rs` exist in the tree during a
// transitional refactor. The included file continues to provide
// `crate::cli::Args` and the public API expected by the rest of the crate.
pub mod cli;
pub mod networking;
// The ip_api module is included inline to avoid ambiguity during a
// transitional refactor when both `src/ip_api.rs` and
// `src/ip_api/mod.rs` might exist. This ensures the crate compiles
// and doc-tests run while we finalize the directory-style layout.
pub mod ip_api {
	include!("ip_api/mod.rs");
}
pub mod metrics;
// The `app` module is implemented in `src/app.rs` and may contain submodules
// in `src/app/*.rs`. To avoid ambiguity between `src/app.rs` and
// `src/app/mod.rs` on some filesystems, include the file here explicitly.
pub mod app {
	include!("app.rs");
}
pub mod xml_io;
// Inline-include the `json_io` module during the refactor to avoid
// duplicate-module doc-test errors when both `src/json_io.rs` and
// `src/json_io/mod.rs` may exist temporarily.
pub mod json_io {
	include!("json_io/mod.rs");
}
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
