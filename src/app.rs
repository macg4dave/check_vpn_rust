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
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};

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

    // Optionally start a tiny HTTP server to serve health and basic metrics.
    // This is intentionally minimal to avoid adding a heavy HTTP dependency.
    let mut metrics_handle: Option<std::thread::JoinHandle<()>> = None;
    if args.enable_metrics {
        let kr_clone = keep_running.clone();
        let addr = args.metrics_addr.clone();
        metrics_handle = Some(std::thread::spawn(move || {
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    log::error!("metrics server failed to bind {}: {}", addr, e);
                    return;
                }
            };
            // Non-blocking accept loop so we can check shutdown flag periodically.
            listener.set_nonblocking(true).ok();
            log::info!("metrics server listening on {}", addr);

            while kr_clone.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _peer)) => {
                        // Read up to 2KB of request; it's fine for simple GET requests.
                        let mut buf = [0u8; 2048];
                        if let Ok(n) = stream.read(&mut buf) {
                            let req = String::from_utf8_lossy(&buf[..n]);
                            if req.starts_with("GET /health") {
                                let resp = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                                let _ = stream.write_all(resp.as_bytes());
                            } else if req.starts_with("GET /metrics") {
                                // A tiny Prometheus-friendly metric
                                let body = "# HELP check_vpn_up 1 if the service is up\n# TYPE check_vpn_up gauge\ncheck_vpn_up 1\n";
                                let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
                                let _ = stream.write_all(resp.as_bytes());
                            } else {
                                let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                                let _ = stream.write_all(resp.as_bytes());
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                    Err(_) => break,
                }
            }
            log::info!("metrics server at {} shutting down", addr);
        }));
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

/// Perform a single connectivity+ISP check using injected dependencies. This is small and
/// easy to unit-test by providing mock closures for `get_isp` and `run_action_fn`.
pub fn perform_check<FGet, FRun>(
    eff: &EffectiveConfig,
    get_isp_fn: FGet,
    run_action_fn: FRun,
) -> Result<()>
where
    FGet: Fn() -> Result<String>,
    FRun: Fn(&actions::Action, bool),
{
    // Convert endpoints into Vec<&str> for call
    let endpoints_ref: Vec<&str> = eff.connectivity_endpoints.iter().map(|s| s.as_str()).collect();

    match networking::is_online_with_retries(&endpoints_ref, eff.connectivity_timeout_secs, &eff.connectivity_ports, eff.connectivity_retries) {
        Ok(true) => {
            match get_isp_fn() {
                Ok(isp) => {
                    if isp == eff.isp_to_check {
                        warn!("VPN Lost (ISP: {})", isp);
                        let action = actions::parse_action(&eff.action_type, &eff.action_arg);
                        run_action_fn(&action, eff.dry_run);
                    } else {
                        info!("VPN active (ISP: {})", isp);
                    }
                }
                Err(e) => {
                    error!("Failed to determine ISP: {}", e);
                    // If this was a single-run invocation or the user requested exit-on-error,
                    // make the failure visible to scripts
                    if eff.run_once || eff.exit_on_error {
                        std::process::exit(crate::config::EXIT_ISP_FAILURE);
                    }
                }
            }
        }
        Ok(false) => {
            error!("Internet appears to be down (connectivity checks failed)");
            if eff.run_once || eff.exit_on_error {
                std::process::exit(crate::config::EXIT_CONNECTIVITY_FAILURE);
            }
        }
        Err(e) => {
            error!("Connectivity check failed: {}", e);
            // If DNS resolution or other networking error occurs and this was a single-run
            // invocation, exit with a specific code to help scripts distinguish failures.
            if eff.run_once || eff.exit_on_error {
                match e {
                    crate::networking::NetworkingError::DnsResolve(_) => {
                        std::process::exit(crate::config::EXIT_CONNECTIVITY_DNS);
                    }
                    _ => {
                        std::process::exit(crate::config::EXIT_CONNECTIVITY_FAILURE);
                    }
                }
            }
        }
    }

    Ok(())
}
