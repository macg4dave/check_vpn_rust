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

    // Delegate to the testable crate-level entrypoint which performs
    // initialization, optional wizard/completions handling, configuration
    // loading and then runs the application. Keeping `try_main` thin like this
    // preserves the binary's semantics while enabling tests to call the same
    // behavior via the library function.

    // Delegate to the testable crate-level entrypoint so tests can exercise
    // the real startup path without running the binary `main` directly.
    // This keeps runtime behavior consistent while enabling integration tests
    // to provide custom `Args` (including non-UTF8 paths).
    check_vpn::try_run_with_args(args)?;
    Ok(())
}
