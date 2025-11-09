use log::error;
use anyhow::Result;
use std::process::ExitCode;

/// Entry point for the binary.
///
/// We keep `main` thin and deterministic: initialization and the bulk of the
/// work is performed in `try_main()` which returns a `Result`. `main` maps
/// that result into an appropriate `ExitCode` and ensures a consistent exit
/// path for tests and embedding environments.
fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            // Log the error (logger should have been set up by `try_main`).
            error!("fatal error running app: {}", e);
            ExitCode::from(1)
        }
    }
}

/// Perform startup and run the application. Returns an error on fatal
/// failure. This function is separate from `main` to make the startup logic
/// testable in isolation (if desired) and to avoid directly calling
/// `std::process::exit` from deep inside the code.
fn try_main() -> Result<()> {
    // Parse CLI and load config (CLI takes precedence when merging within app).
    let args = check_vpn::cli::Args::parse_args();

    // Initialize logging early using CLI verbosity so app.run can log at the
    // requested level. This respects RUST_LOG if already set.
    check_vpn::logging::init_with_verbosity(args.verbose);

    // Try to load config from disk; on error we log and fall back to defaults.
    let cfg = match check_vpn::config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config, using defaults: {}", e);
            check_vpn::config::Config::default()
        }
    };

    // Delegate to the main application logic. `app::run` returns an anyhow::Result
    // so we simply propagate any error upwards for `main` to convert to an
    // appropriate exit code.
    check_vpn::app::run(args, cfg)?;
    Ok(())
}
