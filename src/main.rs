mod cli;
mod networking;
mod actions;
mod logging;
mod config;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use log::{error, info, warn, debug};

use cli::Args;
use config::Config;

fn main() {
    logging::init();
    let args = Args::parse_args();

    // Load XML config (if any) and merge with CLI. CLI options take precedence.
    let cfg = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config, using defaults: {}", e);
            Config::default()
        }
    };

    // Merge: CLI overrides config values; fall back to defaults in Config::default
    let interval = args.interval.or(cfg.interval).unwrap_or(60);
    let isp_to_check = args
        .isp_to_check
        .or(cfg.isp_to_check.clone())
        .unwrap_or_else(|| "Hutchison 3G UK Ltd".to_string());

    let vpn_lost_action_type = args
        .vpn_lost_action_type
        .or(cfg.vpn_lost_action_type.clone())
        .unwrap_or_else(|| "reboot".to_string());

    let vpn_lost_action_arg = args
        .vpn_lost_action_arg
        .or(cfg.vpn_lost_action_arg.clone())
        .unwrap_or_else(|| "/sbin/shutdown -r now".to_string());

    let dry_run = if args.dry_run { true } else { cfg.dry_run.unwrap_or(false) };

    info!("Starting check_vpn (interval={}s, isp_to_check='{}')", interval, isp_to_check);

    // Validate merged configuration before we start the main loop.
    if let Err(e) = config::Config::validate_values(interval, &isp_to_check, &vpn_lost_action_type, &vpn_lost_action_arg) {
        error!("Configuration validation failed: {}", e);
        // Print each reason (split by ';') to make it clear to operators.
        for part in e.to_string().split(';') {
            error!("  - {}", part.trim());
        }
        std::process::exit(2);
    }

    let action = actions::parse_action(&vpn_lost_action_type, &vpn_lost_action_arg);
    debug!("Configured action: {:?}", action);

    let keep_running = Arc::new(AtomicBool::new(true));
    let kr = keep_running.clone();
    ctrlc::set_handler(move || {
        kr.store(false, Ordering::SeqCst);
        info!("Received termination signal, shutting down...");
    }).expect("Error setting Ctrl-C handler");

    while keep_running.load(Ordering::SeqCst) {
        debug!("Running vpn check iteration");
        match networking::get_isp() {
            Ok(isp) => {
                if isp == isp_to_check {
                    warn!("VPN Lost (ISP: {})", isp);
                    actions::run_action(&action, dry_run);
                } else {
                    info!("VPN active (ISP: {})", isp);
                }
            }
            Err(e) => {
                error!("Internet down or failed to determine ISP: {}", e);
            }
        }

        // Sleep but wake earlier if we are asked to stop
        let mut slept = 0u64;
        while slept < interval && keep_running.load(Ordering::SeqCst) {
            let to_sleep = std::cmp::min(1, interval as i32) as u64;
            sleep(Duration::from_secs(to_sleep));
            slept += to_sleep;
        }
    }

    info!("Exiting");
}
