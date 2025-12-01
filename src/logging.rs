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
    // `Write` is only needed for the env_logger formatting below, which is
    // compiled when `feature = "observability"` is NOT enabled. Move the
    // import into that cfg block to avoid unused-import warnings when the
    // observability path is active.

    let level = level_from_verbosity(verbosity);

    // If the optional 'observability' feature is enabled we use the
    // `tracing` / `tracing-subscriber` stack so consumers get structured
    // logging and better span integration. The feature is opt-in and
    // otherwise we keep the existing env_logger behaviour.
    #[cfg(feature = "observability")]
    {
        // Respect existing RUST_LOG if set and otherwise default from verbosity.
        set_rust_log_if_unset(level);

        // Use tracing-subscriber with an env filter and pretty formatting
        // to make logs easier to debug while also supporting structured
        // output for downstream observability systems.
        use tracing_subscriber::{fmt, prelude::*, EnvFilter};

        // Forward `log` macros into the tracing subscriber so existing log! calls
        // continue to emit output when observability is enabled.
        let _ = tracing_log::LogTracer::init();

        let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| level.to_string());
        let filter = EnvFilter::try_new(env_filter).unwrap_or_else(|_| EnvFilter::new(level));

        // Construct a subscriber that prints events with timestamps and the level.
        // We don't set a global panic hook here; the existing hook below will
        // still run and emit errors through `tracing::error!`.
        // Use the crate-provided `SystemTime` timestamp provider by default
        // to avoid enabling optional features such as `chrono` in the
        // tracing-subscriber dependency. This keeps the default build
        // lightweight while still providing timestamps.
        let fmt_layer = fmt::layer()
            .with_timer(fmt::time::time())
            .with_writer(std::io::stderr);

        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .try_init();
    }

    #[cfg(not(feature = "observability"))]
    {
        use env_logger::Env;
        use std::io::Write;

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
    }

    // Install a panic hook that logs panic information. This helps ensure
    // panics are visible even if the default hook is later overridden.
    let _ = std::panic::take_hook();
    std::panic::set_hook(Box::new(|info| {
        // Log panic info at error level with message and optional location.
        // Avoid cloning the payload if possible by logging borrowed references.
        if let Some(s) = info.payload().downcast_ref::<&str>() {
            if let Some(location) = info.location() {
                #[cfg(feature = "observability")]
                tracing::error!(
                    "panic occurred: {} at {}:{}",
                    s,
                    location.file(),
                    location.line()
                );
                #[cfg(not(feature = "observability"))]
                log::error!(
                    "panic occurred: {} at {}:{}",
                    s,
                    location.file(),
                    location.line()
                );
            } else {
                #[cfg(feature = "observability")]
                tracing::error!("panic occurred: {}", s);
                #[cfg(not(feature = "observability"))]
                log::error!("panic occurred: {}", s);
            }
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            if let Some(location) = info.location() {
                #[cfg(feature = "observability")]
                tracing::error!(
                    "panic occurred: {} at {}:{}",
                    s.as_str(),
                    location.file(),
                    location.line()
                );
                #[cfg(not(feature = "observability"))]
                log::error!(
                    "panic occurred: {} at {}:{}",
                    s.as_str(),
                    location.file(),
                    location.line()
                );
            } else {
                #[cfg(feature = "observability")]
                tracing::error!("panic occurred: {}", s.as_str());
                #[cfg(not(feature = "observability"))]
                log::error!("panic occurred: {}", s.as_str());
            }
        } else if let Some(location) = info.location() {
            #[cfg(feature = "observability")]
            tracing::error!(
                "panic occurred: {} at {}:{}",
                "Unknown panic payload",
                location.file(),
                location.line()
            );
            #[cfg(not(feature = "observability"))]
            log::error!(
                "panic occurred: {} at {}:{}",
                "Unknown panic payload",
                location.file(),
                location.line()
            );
        } else {
            #[cfg(feature = "observability")]
            tracing::error!("panic occurred: {}", "Unknown panic payload");
            #[cfg(not(feature = "observability"))]
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

    // Helpers to mutate env in a deterministic fashion for tests.
    fn remove_rust_log() {
        std::env::remove_var("RUST_LOG");
    }
    fn set_rust_log(v: &str) {
        std::env::set_var("RUST_LOG", v);
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
