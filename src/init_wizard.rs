use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::{self, Write};
use std::io::IsTerminal;
// std::fs not needed; writing via xml_io

use crate::config::Config;
use crate::providers;
use crate::providers::VpnInfoProvider;

// Test-only override: when set to true, disable prompts regardless of TTY/env.
static NO_PROMPT_OVERRIDE: AtomicBool = AtomicBool::new(false);

/// For tests: disable interactive prompts regardless of TTY/ENV.
/// This is a no-op in release usage unless called by tests.
pub fn set_no_prompt_for_tests(disable: bool) {
    NO_PROMPT_OVERRIDE.store(disable, Ordering::Relaxed);
}

pub fn run_init(output: Option<String>, no_fetch: bool) -> Result<()> {
    println!("check_vpn init — generate a config.xml");

    // Decide interactive vs non-interactive:
    // Prompts are only shown when stdin is a TTY AND no disabling env var is set.
    // Setting CHECK_VPN_INIT_NO_PROMPT=1 (or running under common CI env markers) disables interaction.
    let ci_env = std::env::var("CI").ok().is_some();
    let disable_env = std::env::var("CHECK_VPN_INIT_NO_PROMPT").ok().is_some();
    let interactive = io::stdin().is_terminal() && !ci_env && !disable_env && !NO_PROMPT_OVERRIDE.load(Ordering::Relaxed);

    // Determine ISP via providers unless skipped
    let mut detected_isp = String::from("Your ISP Here");
    if !no_fetch {
        let mut chain: Vec<Box<dyn VpnInfoProvider>> = Vec::new();
        if let Ok(p) = providers::ip_api_provider::IpApiProvider::new_default() { chain.push(Box::new(p)); }
        if let Ok(p) = providers::ifconfig_co_provider::IfconfigCoProvider::new_default() { chain.push(Box::new(p)); }
        // Apply a hard timeout (20s) around provider querying so tests / CI never hang indefinitely.
    // Move chain into closure to satisfy 'static requirement.
    let chain_for_closure = chain;
    match run_with_timeout(move || providers::query_first_success(&chain_for_closure), std::time::Duration::from_secs(20)) {
            Ok(Ok(id)) => detected_isp = id.isp,
            Ok(Err(e)) => eprintln!("ISP detection failed: {e}"),
            Err(_) => eprintln!("ISP detection timed out after 20s; continuing with placeholder"),
        }
    }

    // Prompt minimal fields
    let isp = prompt_default(interactive, "ISP to watch for (vpn lost)", &detected_isp)?;
    let interval = prompt_default(interactive, "Interval seconds", "60")?;
    let action = prompt_default(interactive, "Action type (reboot|restart-unit|command)", "restart-unit")?;
    let action_arg = match action.as_str() {
        "restart-unit" => prompt_default(interactive, "Systemd unit to restart", "openvpn-client@myvpn.service")?,
        "command" => prompt_default(interactive, "Command to run", "/usr/local/bin/reconnect_vpn.sh")?,
        _ => String::new(),
    };

    // Build Config
    let cfg = Config {
        interval: Some(interval.parse().unwrap_or(60)),
        isp_to_check: Some(isp),
        vpn_lost_action_type: Some(action),
        vpn_lost_action_arg: Some(action_arg),
        dry_run: Some(false),
        exit_on_error: Some(false),
        connectivity_endpoints: Some(vec!["8.8.8.8".to_string(), "google.com".to_string()]),
        connectivity_ports: Some(vec![443, 53]),
        connectivity_timeout_secs: Some(crate::networking::DEFAULT_TIMEOUT_SECS),
        connectivity_retries: Some(crate::networking::DEFAULT_RETRIES),
        enable_ip_api: Some(true),
        enable_ifconfig_co: Some(true),
        provider_urls: Some(vec![]),
        custom_json_server: None,
        custom_json_key: None,
    };

    let out_path = output.unwrap_or_else(|| "./check_vpn.xml".to_string());
    crate::xml_io::write_xml(&cfg, &out_path)?;
    println!("Wrote {}", out_path);
    Ok(())
}

fn prompt_default(interactive: bool, label: &str, default: &str) -> Result<String> {
    if !interactive {
        return Ok(default.to_string());
    }
    print!("{} [{}]: ", label, default);
    io::stdout().flush()?;
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    let v = buf.trim();
    Ok(if v.is_empty() { default.to_string() } else { v.to_string() })
}

// Simple timeout helper: executes closure on a thread and joins with timeout.
fn run_with_timeout<F, R>(f: F, dur: std::time::Duration) -> Result<R, ()>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    use std::thread;
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(f());
    });
    match rx.recv_timeout(dur) {
        Ok(r) => Ok(r),
        Err(_) => Err(()),
    }
}
