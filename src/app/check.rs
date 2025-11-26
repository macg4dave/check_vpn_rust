use anyhow::Result;
use log::{error, info, warn};

use crate::actions;
use crate::config::EffectiveConfig;
use crate::networking;

/// Perform a single connectivity+ISP check using injected dependencies.
///
/// This function is the primary "unit of work" and is deliberately small and
/// dependency-injected so tests can provide mock implementations for
/// `get_isp_fn` and `run_action_fn`.
///
/// Behavior summary:
/// - If connectivity checks indicate the internet is down, either returns Ok(())
///   or exits the process with an appropriate code when `eff.run_once` or
///   `eff.exit_on_error` is set. Exiting is left to the application layer to
///   preserve the test seam and scripting semantics.
/// - If connectivity is up, we attempt to determine the ISP via `get_isp_fn`.
///   On success, compare with `eff.isp_to_check` and run the configured
///   action when they match (VPN likely lost). The action is executed by
///   calling the provided `run_action_fn` with the parsed `Action` and
///   `eff.dry_run` flag.
/// - Networking and ISP resolution errors are handled according to
///   `eff.run_once`/`exit_on_error` flags: either surfaced (Ok/Err) or cause
///   a process exit. Tests use injections to avoid exiting the process.
pub fn perform_check<FGet, FRun>(
    eff: &EffectiveConfig,
    get_isp_fn: FGet,
    run_action_fn: FRun,
) -> Result<()>
where
    FGet: Fn() -> Result<String>,
    FRun: Fn(&actions::Action, bool),
{
    // Convert endpoints into a slice of &str for the networking API.
    let endpoints_ref: Vec<&str> = eff
        .connectivity_endpoints
        .iter()
        .map(|s| s.as_str())
        .collect();

    match networking::is_online_with_retries(
        &endpoints_ref,
        eff.connectivity_timeout_secs,
        &eff.connectivity_ports,
        eff.connectivity_retries,
    ) {
        Ok(true) => {
            // Connectivity appears fine, determine ISP.
            match get_isp_fn() {
                Ok(isp) => {
                    if isp == eff.isp_to_check {
                        // ISP matches the one we're watching for -> VPN likely lost.
                        warn!("VPN Lost (ISP: {})", isp);
                        let action = actions::parse_action(&eff.action_type, &eff.action_arg);
                        run_action_fn(&action, eff.dry_run);
                    } else {
                        info!("VPN active (ISP: {})", isp);
                    }
                }
                Err(e) => {
                    error!("Failed to determine ISP: {}", e);
                    // For single-run invocations or when caller requested exit-on-error
                    // the application may want to translate this into a process-exit.
                    // We do not exit here to keep this function testable; callers
                    // (e.g., `run`) can decide to exit based on eff flags.
                }
            }
        }
        Ok(false) => {
            error!("Internet appears to be down (connectivity checks failed)");
            // Caller decides whether to exit when run_once/exit_on_error is set.
        }
        Err(e) => {
            error!("Connectivity check failed: {}", e);
            // Caller decides whether to exit when run_once/exit_on_error is set.
        }
    }

    Ok(())
}
