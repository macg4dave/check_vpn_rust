use crate::error_types::AppError;
use std::result::Result;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::actions;
use crate::cli::Args;
use crate::config::Config;
use crate::providers::{self, VpnInfoProvider};
// TcpStream previously used to wake a metrics listener; no longer required here.

mod check {
    include!("app/check.rs");
}

pub use check::perform_check;

/// Run the main application logic. Returns Ok(()) on clean shutdown, or Err on fatal error.
pub fn run(args: Args, cfg: Config) -> Result<(), AppError> {
    info!("Starting check_vpn run()");
    debug!(
        "Starting run with config override={}",
        args.config
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(none)".to_string())
    );

    // Start with the provided config merged with CLI args.
    let mut current_cfg = cfg;
    let mut eff = current_cfg.merge_with_args(&args);

    // Validate merged configuration before we start the main loop.
    if let Err(e) = crate::config::Config::validate_values(
        eff.interval,
        &eff.isp_to_check,
        &eff.action_type,
        &eff.action_arg,
        &eff.connectivity_endpoints,
        &eff.connectivity_ports,
        eff.connectivity_timeout_secs,
        eff.connectivity_retries,
    ) {
        error!("Configuration validation failed: {:?}", e);
        for part in (e.0).iter() {
            error!("  - {}", part.trim());
        }
        return Err(AppError::ConfigError(e));
    }

    let action = actions::parse_action(&eff.action_type, &eff.action_arg);
    debug!("Configured action: {:?}", action);

    // If run_once requested, perform single check and exit. Use perform_check directly (testable).
    if eff.run_once {
        let get_isp_closure = build_provider_chain_getter(&eff);
        perform_check(&eff, get_isp_closure, actions::run_action)?;
        return Ok(());
    }

    let keep_running = Arc::new(AtomicBool::new(true));
    let kr = keep_running.clone();
    ctrlc::set_handler(move || {
        kr.store(false, Ordering::SeqCst);
        info!("Received termination signal, shutting down...");
    })
    .map_err(|e| AppError::RuntimeError(anyhow::anyhow!("failed to set ctrlc handler: {:?}", e)))?;

    // Metrics server is optional; CLI flags are not present in this build,
    // so keep a placeholder handle for symmetry with earlier implementations.
    let metrics_handle: Option<std::thread::JoinHandle<()>> = None;

    // Main loop: cross-platform timer/polling. On each iteration we attempt to reload
    // the XML config and merge with CLI args so users can change timing/config without
    // restarting the service. This is simpler and portable across Linux/Windows.
    while keep_running.load(Ordering::SeqCst) {
        debug!("Running vpn check iteration");

        // Try to reload config from disk; if successful and changed, merge and validate.
        // Prefer the explicit CLI --config path (if provided) so reloads use the
        // same file that was used at startup.
        let loaded = if let Some(path) = &args.config {
            crate::config::Config::load_from_path(path.to_str().unwrap())
        } else {
            crate::config::Config::load()
        };
        match loaded {
            Ok(new_cfg) => {
                if new_cfg != current_cfg {
                    debug!("Config file changed on disk, merging new values");
                    let new_eff = new_cfg.merge_with_args(&args);
                    match crate::config::Config::validate_values(
                        new_eff.interval,
                        &new_eff.isp_to_check,
                        &new_eff.action_type,
                        &new_eff.action_arg,
                        &new_eff.connectivity_endpoints,
                        &new_eff.connectivity_ports,
                        new_eff.connectivity_timeout_secs,
                        new_eff.connectivity_retries,
                    ) {
                        Ok(()) => {
                            current_cfg = new_cfg;
                            eff = new_eff;
                            debug!("Applied new effective config: interval={}", eff.interval);
                        }
                        Err(errs) => {
                            error!("New configuration is invalid, keeping previous config:");
                            for part in (errs.0).iter() {
                                error!("  - {}", part.trim());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if args.strict_config {
                    // Emit a clear marker on stderr so external test harnesses can
                    // reliably detect the strict-config exit condition, then exit
                    // early with a non-zero status (tests only assert non-zero).
                    eprintln!("--strict-config is set");
                    eprintln!("Failed to reload config from disk and --strict-config is set; exiting: {:?}", e);
                    std::process::exit(1);
                }
                debug!(
                    "Failed to reload config from disk (path = {}, will continue with previous): {:?}; to fail on reload problems, start with --strict-config",
                    args.config
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .or_else(|| std::env::var("CHECK_VPN_CONFIG").ok())
                        .unwrap_or_else(|| "(no explicit config path set; looked up defaults)".to_string()),
                    e
                );
            }
        }

    // Execute the single check using the current effective configuration.
    let get_isp_closure = build_provider_chain_getter(&eff);
    perform_check(&eff, get_isp_closure, actions::run_action)?;

        // Sleep but wake earlier if we are asked to stop; use the possibly-updated interval.
        let mut slept = 0u64;
        while slept < eff.interval && keep_running.load(Ordering::SeqCst) {
            let to_sleep = std::cmp::min(1, eff.interval as i32) as u64;
            sleep(Duration::from_secs(to_sleep));
            slept += to_sleep;
        }
    }

    info!("Exiting check_vpn run loop");

    // Ensure metrics server exits cleanly if present.
    if let Some(h) = metrics_handle {
        let _ = h.join();
    }
    Ok(())
}

// perform_check has been moved to `src/app_check.rs` and is re-exported above.

/// Build a closure `Fn() -> Result<String>` that queries the configured providers
/// in order and returns the first successful ISP string.
fn build_provider_chain_getter(eff: &crate::config::EffectiveConfig) -> impl Fn() -> anyhow::Result<String> + Send + Sync + 'static {
    let mut chain: Vec<Box<dyn VpnInfoProvider>> = Vec::new();

    // Custom provider URLs first, in specified order
    for url in &eff.provider_urls {
        if let Ok(p) = providers::generic_json_provider::GenericJsonProvider::new(url) {
            chain.push(Box::new(p));
        }
    }

    // Single custom JSON server with optional key override before others.
        if let Some(ref url) = eff.custom_json_server {
        if let Ok(p) = providers::generic_json_provider::GenericJsonProvider::new(url)
            .map(|gp| gp.with_key(eff.custom_json_key.as_deref())) {
            chain.insert(0, Box::new(p));
        }
    }

    // Built-ins
    if eff.enable_ip_api {
        if let Ok(p) = providers::ip_api_provider::IpApiProvider::new_default() {
            chain.push(Box::new(p));
        }
    }
    if eff.enable_ifconfig_co {
        if let Ok(p) = providers::ifconfig_co_provider::IfconfigCoProvider::new_default() {
            chain.push(Box::new(p));
        }
    }

    move || {
        let id = providers::query_first_success(&chain)?;
        Ok(id.isp)
    }
}
