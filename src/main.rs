use anyhow::Context;
use check_vpn::error_types::{exit_code_for_error, AppError};
use log::error;
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
            // Log a debug representation of the error, which helps show
            // chained causes and captures more context.
            error!("fatal error running app: {:?}", e);
            exit_code_for_error(&e)
        }
    }
}

/// Perform startup and run the application. Returns an error on fatal
/// failure. This function is separate from `main` to make the startup logic
/// testable in isolation (if desired) and to avoid directly calling
/// `std::process::exit` from deep inside the code.
fn try_main() -> std::result::Result<(), AppError> {
    // Parse CLI and load config (CLI takes precedence when merging within app).
    let args = check_vpn::cli::Args::parse_args();

    // Initialize logging early using CLI verbosity so app.run can log at the
    // requested level. This respects RUST_LOG if already set.
    check_vpn::logging::init_with_verbosity(args.verbose);

    // If wizard requested, run it and exit.
    if args.init {
        check_vpn::init_wizard::run_init(args.init_output.as_deref(), args.init_no_fetch)
            .map_err(AppError::RuntimeError)?;
        return Ok(());
    }

    // Generate shell completions when requested and exit.
    if let Some(shell) = args.generate_completions.as_deref() {
        check_vpn::completions::generate_completions(shell, args.completions_output.as_deref())
            .map_err(AppError::RuntimeError)?;
        return Ok(());
    }

    // Try to load config from disk; on error we return a typed `AppError` so the
    // binary can map it to a useful exit code.
    // Add context to configuration loading and convert into an application
    // error with the chain preserved in logs. We call `context` so the
    // underlying error has an additional message for troubleshooting.
    let cfg = if let Some(path) = args.config.as_ref() {
        match check_vpn::config::Config::load_from_path(path.to_str().unwrap())
            .context("loading configuration from --config path")
        {
            Ok(c) => c,
            Err(e) => {
                // Try to include a hint about which config path (if any) was being used
                let cfg_path = args
                    .config
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .or_else(|| std::env::var("CHECK_VPN_CONFIG").ok())
                    .unwrap_or_else(|| {
                        "(no explicit config path set; looked up defaults)".to_string()
                    });

                error!(
                        "Failed to load config (path = {}): {:?}. If this is unintended, fix the XML or use --config to specify a different file.",
                        cfg_path,
                        e
                    );
                return Err(AppError::ConfigLoadError(e));
            }
        }
    } else {
        match check_vpn::config::Config::load().context("loading configuration via lookup") {
            Ok(c) => c,
            Err(e) => {
                // Try to include a hint about which config path (if any) was being used
                let cfg_path = args
                    .config
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .or_else(|| std::env::var("CHECK_VPN_CONFIG").ok())
                    .unwrap_or_else(|| {
                        "(no explicit config path set; looked up defaults)".to_string()
                    });

                error!(
                        "Failed to load config (path = {}): {:?}. If this is unintended, fix the XML or use --config to specify a different file.",
                        cfg_path,
                        e
                    );
                return Err(AppError::ConfigLoadError(e));
            }
        }
    };

    // Delegate to the main application logic. `app::run` returns an anyhow::Result
    // so we simply propagate any error upwards for `main` to convert to an
    // appropriate exit code.
    check_vpn::app::run(args, cfg)?;
    Ok(())
}
