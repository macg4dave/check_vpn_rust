use check_vpn::error_types::exit_code_for_error;
use std::process::ExitCode;

fn main() -> ExitCode {
    match check_vpn::try_run_with_args(check_vpn::cli::Args::parse_args()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("fatal error running app: {error:?}");
            exit_code_for_error(&error)
        }
    }
}
