//! Logging helpers.
//!
//! Provides a tiny, testable wrapper to initialize `env_logger` with a
//! sensible default controlled by a `verbosity` parameter. If the
//! environment variable `RUST_LOG` is already set we respect it; otherwise
//! we set a default derived from `verbosity` before initializing the logger.

/// Convert a numeric verbosity level into a `RUST_LOG` style string.
///
/// Mapping:
/// - 0 => "info"
/// - 1 => "debug"
/// - 2+ => "trace"
pub fn level_from_verbosity(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    }
}

/// Set `RUST_LOG` to `level` but only if it's not already present.
///
/// Historically some unusual targets exposed environment helpers as `unsafe`.
/// In current stable Rust `std::env::set_var` and `remove_var` are safe. We
/// therefore use the safe API here so callers (including tests) don't need to
/// rely on `unsafe` blocks.
fn set_rust_log_if_unset(level: &str) {
    if std::env::var_os("RUST_LOG").is_none() {
        // Some cross-compilation targets or older toolchains expose
        // environment mutation as `unsafe`. Keep this call inside an
        // `unsafe` block so the module remains compatible across targets.
        unsafe {
            std::env::set_var("RUST_LOG", level);
        }
    }
}

/// Initialize logging using `env_logger`.
///
/// If `RUST_LOG` is already set it is respected. Otherwise the provided
/// `verbosity` determines the default level (see `level_from_verbosity`).
/// This uses `env_logger::try_init()` to avoid panicking if the logger was
/// already initialized by another test or library; failures are ignored.
pub fn init_with_verbosity(verbosity: u8) {
    let level = level_from_verbosity(verbosity);
    set_rust_log_if_unset(level);
    // try_init returns Err if a logger is already installed; that's fine for
    // our purposes (tests or external embedding). Ignore the error.
    let _ = env_logger::try_init();
}

/// Backwards-compatible default initializer (info level unless RUST_LOG set).
pub fn init() {
    init_with_verbosity(0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    // Helpers to mutate env in a way compatible with targets that mark these
    // functions unsafe. Tests wrap calls in `unsafe` to mirror production.
    fn remove_rust_log() {
        unsafe {
            std::env::remove_var("RUST_LOG");
        }
    }
    fn set_rust_log(v: &str) {
        unsafe {
            std::env::set_var("RUST_LOG", v);
        }
    }

    #[test]
    fn test_level_from_verbosity() {
        assert_eq!(level_from_verbosity(0), "info");
        assert_eq!(level_from_verbosity(1), "debug");
        assert_eq!(level_from_verbosity(2), "trace");
        assert_eq!(level_from_verbosity(99), "trace");
    }

    #[test]
    #[serial]
    fn test_init_sets_rust_log_when_unset() {
        remove_rust_log();
        init_with_verbosity(1);
        let got = std::env::var("RUST_LOG").unwrap_or_default();
        assert!(got.contains("debug") || got == "debug");
        // cleanup
        remove_rust_log();
    }

    #[test]
    #[serial]
    fn test_init_respects_existing_rust_log() {
        remove_rust_log();
        set_rust_log("warn");
        init_with_verbosity(0);
        let got = std::env::var("RUST_LOG").unwrap_or_default();
        assert_eq!(got, "warn");
        remove_rust_log();
    }
}
