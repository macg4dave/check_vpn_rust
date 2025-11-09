use anyhow::Result;
use log::{debug, error, info, warn};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::actions;
use crate::cli::Args;
use crate::config::{Config, EffectiveConfig};
use crate::networking;
use crate::ip_api;
use std::net::TcpStream;

mod check {
    include!("app/check.rs");
}

pub use check::perform_check;

/// Run the main application logic. Returns Ok(()) on clean shutdown, or Err on fatal error.
pub fn run(args: Args, cfg: Config) -> Result<()> {
    info!("Starting check_vpn run()");

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
        error!("Configuration validation failed: {}", e);
        for part in (e.0).iter() {
            error!("  - {}", part.trim());
        }
        std::process::exit(crate::config::EXIT_INVALID_CONFIG);
    }

    let action = actions::parse_action(&eff.action_type, &eff.action_arg);
    debug!("Configured action: {:?}", action);

    // If run_once requested, perform single check and exit. Use perform_check directly (testable).
    if eff.run_once {
        perform_check(&eff, || ip_api::get_isp(), |a, d| actions::run_action(a, d))?;
        return Ok(());
    }

    let keep_running = Arc::new(AtomicBool::new(true));
    let kr = keep_running.clone();
    ctrlc::set_handler(move || {
        kr.store(false, Ordering::SeqCst);
        info!("Received termination signal, shutting down...");
    })?;

    // Optional metrics server (moved to `src/metrics.rs`).
    let mut metrics_handle: Option<std::thread::JoinHandle<()>> = None;
    if args.enable_metrics {
        metrics_handle = crate::metrics::start_metrics_server(&args.metrics_addr, keep_running.clone());
    }

    // Main loop: cross-platform timer/polling. On each iteration we attempt to reload
    // the XML config and merge with CLI args so users can change timing/config without
    // restarting the service. This is simpler and portable across Linux/Windows.
    while keep_running.load(Ordering::SeqCst) {
        debug!("Running vpn check iteration");

        // Try to reload config from disk; if successful and changed, merge and validate.
        match crate::config::Config::load() {
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
                debug!("Failed to reload config from disk (will continue with previous): {}", e);
            }
        }

        // Execute the single check using the current effective configuration.
        perform_check(&eff, || ip_api::get_isp(), |a, d| actions::run_action(a, d))?;

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
        // Wake the listener in case it's blocked in accept
        let _ = TcpStream::connect(args.metrics_addr.replace("http://", ""));
        let _ = h.join();
    }
    Ok(())
}

// perform_check has been moved to `src/app_check.rs` and is re-exported above.
