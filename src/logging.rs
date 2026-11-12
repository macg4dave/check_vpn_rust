//! Process-wide logging initialization without changing the environment or panic hook.

/// Map CLI verbosity to the default filter when `RUST_LOG` is absent.
pub fn level_from_verbosity(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    }
}

fn filter_directives(verbosity: u8, configured: Option<&str>) -> &str {
    configured.unwrap_or_else(|| level_from_verbosity(verbosity))
}

/// Initialize logging, respecting `RUST_LOG` and an already-installed logger.
///
/// Invalid tracing directives fall back to CLI verbosity. Initialization leaves
/// the process environment and the caller's panic hook unchanged.
pub fn init_with_verbosity(verbosity: u8) {
    let configured = std::env::var("RUST_LOG").ok();
    let directives = filter_directives(verbosity, configured.as_deref());

    #[cfg(feature = "observability")]
    {
        use tracing_subscriber::{fmt, prelude::*, EnvFilter};

        let filter = EnvFilter::try_new(directives)
            .unwrap_or_else(|_| EnvFilter::new(level_from_verbosity(verbosity)));
        // try_init installs the log bridge and tolerates an existing subscriber.
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_writer(std::io::stderr))
            .try_init();
    }

    #[cfg(not(feature = "observability"))]
    {
        let mut builder = env_logger::Builder::new();
        builder.parse_filters(directives);
        builder.format_timestamp_millis();
        builder.format_target(true);
        let _ = builder.try_init();
    }
}

/// Initialize logging with info verbosity unless overridden by `RUST_LOG`.
pub fn init() {
    init_with_verbosity(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verbosity_selects_default_filter() {
        for (verbosity, expected) in [(0, "info"), (1, "debug"), (2, "trace"), (99, "trace")] {
            assert_eq!(filter_directives(verbosity, None), expected);
        }
    }

    #[test]
    fn configured_filter_takes_precedence() {
        assert_eq!(
            filter_directives(2, Some("check_vpn=warn")),
            "check_vpn=warn"
        );
        assert_eq!(filter_directives(2, Some("")), "");
    }
}
