/// Simple logging setup wrapper. Keeps `main.rs` tidy and centralizes logging initialization.
///
/// If `RUST_LOG` is already set it is respected. Otherwise, `verbosity` controls
/// the default level: 0 => info, 1 => debug, 2+ => trace.
pub fn init_with_verbosity(verbosity: u8) {
    if std::env::var_os("RUST_LOG").is_none() {
        let level = match verbosity {
            0 => "info",
            1 => "debug",
            _ => "trace",
        };
        std::env::set_var("RUST_LOG", level);
    }
    env_logger::init();
}

/// Backwards-compatible default initializer (info level unless RUST_LOG set).
pub fn init() {
    init_with_verbosity(0);
}
