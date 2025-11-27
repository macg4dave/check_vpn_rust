//! Logging helpers.
//!
//! Provides a tiny, testable wrapper to initialize `env_logger` with a
//! sensible default controlled by a `verbosity` parameter. If the
//! environment variable `RUST_LOG` is already set we respect it; otherwise
//! we set a default derived from `verbosity` before initializing the
//! logger. This module also configures a panic hook that logs panics
//! at the error level so panics are visible in structured logs.

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
        // No RUST_LOG set: set a sensible default based on verbosity.
        std::env::set_var("RUST_LOG", level);
    }
}

/// Initialize logging using `env_logger`.
///
/// If `RUST_LOG` is already set it is respected. Otherwise the provided
/// `verbosity` determines the default level (see `level_from_verbosity`).
/// This uses `env_logger::try_init()` to avoid panicking if the logger was
/// already initialized by another test or library; failures are ignored.
pub fn init_with_verbosity(verbosity: u8) {
    use env_logger::Env;
    use std::io::Write;

    let level = level_from_verbosity(verbosity);
    set_rust_log_if_unset(level);

    // Build a formatted logger that includes timestamp and level. If the
    // environment variable RUST_LOG is set, that will take precedence.
    let env = Env::default().default_filter_or(level);
    let mut builder = env_logger::Builder::from_env(env);
    builder.format(|buf, record| {
        writeln!(
            buf,
            "{} {:<5} [{}] {}",
            buf.timestamp_millis(),
            record.level(),
            record.target(),
            record.args()
        )
    });
    let _ = builder.try_init();

    // Install a panic hook that logs panic information. This helps ensure
    // panics are visible even if the default hook is later overridden.
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        // Log panic info at error level with message and optional location.
        // Avoid cloning the payload if possible by logging borrowed references.
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            if let Some(location) = info.location() {
                log::error!("panic occurred: {} at {}:{}", s, location.file(), location.line());
            } else {
                log::error!("panic occurred: {}", s);
            }
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            if let Some(location) = info.location() {
                log::error!("panic occurred: {} at {}:{}", s.as_str(), location.file(), location.line());
            } else {
                log::error!("panic occurred: {}", s.as_str());
            }
        } else if let Some(location) = info.location() {
            log::error!("panic occurred: {} at {}:{}", "Unknown panic payload", location.file(), location.line());
        } else {
            log::error!("panic occurred: {}", "Unknown panic payload");
        }
    }));
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
