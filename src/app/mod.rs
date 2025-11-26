use anyhow::Result;
use log::{debug, error, info};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::actions;
use crate::cli::Args;
use crate::config::Config;
use crate::ip_api;

mod check;

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
        perform_check(&eff, ip_api::get_isp, actions::run_action)?;
        return Ok(());
    }

    let keep_running = Arc::new(AtomicBool::new(true));
    let kr = keep_running.clone();
    ctrlc::set_handler(move || {
        kr.store(false, Ordering::SeqCst);
        info!("Received termination signal, shutting down...");
    })?;

    // Metrics server removed: out of scope for this build.

    debug!("Starting main check loop (interval = {} sec)", eff.interval);

    while keep_running.load(Ordering::SeqCst) {
        // Check for config file updates unless run_once mode. This allows
        // configuration hot-reloading without restarting the service. Load a new
        // config, merge with args, and validate. On any error, emit a warning
        // and continue with the previous settings.
        if !eff.run_once {
            match Config::load() {
                Ok(new_config) => {
                    let new_eff = new_config.merge_with_args(&args);
                    // We should validate the new config before using it.
                    if let Err(e) = crate::config::Config::validate_values(
                        new_eff.interval,
                        &new_eff.isp_to_check,
                        &new_eff.action_type,
                        &new_eff.action_arg,
                        &new_eff.connectivity_endpoints,
                        &new_eff.connectivity_ports,
                        new_eff.connectivity_timeout_secs,
                        new_eff.connectivity_retries,
                    ) {
                        error!("Reloaded config failed validation, keeping previous: {}", e);
                    } else {
                        // Update both the base config and effective merged config
                        current_cfg = new_config;
                        eff = new_eff;
                        // Suppress unused variable warning - current_cfg is kept for potential future use
                        let _ = &current_cfg;
                        debug!("Reloaded configuration successfully");
                        if eff.run_once {
                            debug!("Config now has run_once=true, will exit after this iteration");
                        }
                    }
                }
                Err(e) => {
                    debug!(
                        "Failed to reload config from disk (will continue with previous): {}",
                        e
                    );
                }
            }
        }

        // Execute the single check using the current effective configuration.
        perform_check(&eff, ip_api::get_isp, actions::run_action)?;

        // Sleep but wake earlier if we are asked to stop; use the possibly-updated interval.
        let mut slept = 0u64;
        while slept < eff.interval && keep_running.load(Ordering::SeqCst) {
            let to_sleep = std::cmp::min(1, eff.interval as i32) as u64;
            sleep(Duration::from_secs(to_sleep));
            slept += to_sleep;
        }
    }

    info!("Exiting check_vpn run loop");

    // no-op: metrics server removed
    Ok(())
}
