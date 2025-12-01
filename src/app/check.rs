use anyhow::Result as AnyhowResult;
use crate::error_types::AppError;
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
)-> Result<(), AppError>
where
    FGet: Fn() -> AnyhowResult<String>,
    FRun: Fn(&actions::Action, bool) -> AnyhowResult<()>,
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
                        if let Err(e) = run_action_fn(&action, eff.dry_run) {
                            error!("Action execution failed: {:?}", e);
                            return Err(AppError::ActionFailed(e));
                        }
                    } else {
                        info!("VPN active (ISP: {})", isp);
                    }
                }
                Err(e) => {
                    error!("Failed to determine ISP: {:?}", e);
                    // For single-run invocations or when caller requested exit-on-error
                    // the application may want to translate this into a process-exit.
                    // We do not exit here to keep this function testable; callers
                    // (e.g., `run`) can decide to exit based on eff flags.
                    if eff.run_once || eff.exit_on_error {
                        return Err(AppError::RuntimeError(e));
                    }
                }
            }
        }
        Ok(false) => {
            error!("Internet appears to be down (connectivity checks failed)");
            // Caller decides whether to exit when run_once/exit_on_error is set.
            if eff.run_once || eff.exit_on_error {
                return Err(AppError::RuntimeError(anyhow::anyhow!(
                    "connectivity check failed (offline)",
                )));
            }
        }
            Err(e) => {
            error!("Connectivity check failed: {:?}", e);
            // Caller decides whether to exit when run_once/exit_on_error is set.
                    if eff.run_once || eff.exit_on_error {
                        return Err(AppError::RuntimeError(e.into()));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EffectiveConfig;
    use anyhow::anyhow;
    use crate::error_types::AppError;

    #[test]
    fn perform_check_returns_runtime_error_with_context_when_isp_fails() {
        // Create a local listener so connectivity checks report 'online'.
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();

        let eff = EffectiveConfig {
            interval: 1,
            isp_to_check: "X".to_string(),
            action_type: "reboot".to_string(),
            action_arg: "".to_string(),
            dry_run: true,
            connectivity_endpoints: vec![format!("127.0.0.1:{}", port)],
            connectivity_ports: vec![443],
            connectivity_timeout_secs: 1,
            connectivity_retries: 1,
            run_once: true,
            exit_on_error: true,
            enable_ip_api: false,
            enable_ifconfig_co: false,
            provider_urls: vec![],
            custom_json_server: None,
            custom_json_key: None,
        };

        // get_isp_fn returns a controlled anyhow error
        let get_isp = || -> anyhow::Result<String> { Err(anyhow!("simulated provider fail")) };
        let run_action = |_a: &crate::actions::Action, _d: bool| -> anyhow::Result<()> { Ok(()) };

        let res = perform_check(&eff, get_isp, run_action);
        assert!(res.is_err());
        if let Err(err) = res {
            // AppError::RuntimeError should contain the debug string with underlying error
            assert!(err.to_string().contains("simulated provider fail"));
        }
    }

    #[test]
    fn action_failure_is_propagated_as_app_error() {
        // Create a local listener so connectivity reports online.
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();

        let eff = EffectiveConfig {
            interval: 1,
            isp_to_check: "TestISP".to_string(),
            action_type: "reboot".to_string(),
            action_arg: "".to_string(),
            dry_run: true,
            connectivity_endpoints: vec![format!("127.0.0.1:{}", port)],
            connectivity_ports: vec![port],
            connectivity_timeout_secs: 1,
            connectivity_retries: 1,
            run_once: true,
            exit_on_error: true,
            enable_ip_api: false,
            enable_ifconfig_co: false,
            provider_urls: vec![],
            custom_json_server: None,
            custom_json_key: None,
        };

        let get_isp = || -> anyhow::Result<String> { Ok("TestISP".to_string()) };
        let failing_action = |_a: &crate::actions::Action, _d: bool| -> anyhow::Result<()> {
            Err(anyhow!("simulated action failure"))
        };

        let res = perform_check(&eff, get_isp, failing_action);
        assert!(
            matches!(res, Err(AppError::ActionFailed(_))),
            "expected AppError::ActionFailed when action execution fails"
        );
    }
}
