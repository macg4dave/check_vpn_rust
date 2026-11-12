use anyhow::{anyhow, Context, Result};
use std::io::IsTerminal;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::config::Config;
use crate::providers;

// Test-only override: when set to true, disable prompts regardless of TTY/env.
static NO_PROMPT_OVERRIDE: AtomicBool = AtomicBool::new(false);

/// For tests: disable interactive prompts regardless of TTY/ENV.
/// This is a no-op in release usage unless called by tests.
pub fn set_no_prompt_for_tests(disable: bool) {
    NO_PROMPT_OVERRIDE.store(disable, Ordering::Relaxed);
}

pub fn run_init(output: Option<&str>, no_fetch: bool) -> Result<()> {
    println!("check_vpn init - generate a config.xml");

    // Decide interactive vs non-interactive:
    // Prompts are only shown when stdin is a TTY AND no disabling env var is set.
    // Setting CHECK_VPN_INIT_NO_PROMPT=1 (or running under common CI env markers) disables interaction.
    let ci_env = std::env::var("CI").ok().is_some();
    let disable_env = std::env::var("CHECK_VPN_INIT_NO_PROMPT").ok().is_some();
    let interactive = io::stdin().is_terminal()
        && !ci_env
        && !disable_env
        && !NO_PROMPT_OVERRIDE.load(Ordering::Relaxed);

    // Determine ISP via providers unless skipped
    let mut detected_isp = String::from("Your ISP Here");
    if !no_fetch {
        let chain_for_closure = providers::build_chain(&crate::config::EffectiveConfig::default())?;
        match run_with_timeout(
            move |cancel| providers::query_first_success_with_cancel(&chain_for_closure, &cancel),
            std::time::Duration::from_secs(20),
        ) {
            Ok(Ok(id)) => detected_isp = id.isp,
            Ok(Err(e)) => eprintln!("ISP detection failed: {e}"),
            Err(_) => eprintln!("ISP detection timed out after 20s; continuing with placeholder"),
        }
    }

    // Prompt minimal fields
    let isp = prompt_default(interactive, "ISP to watch for (vpn lost)", &detected_isp)?;
    let interval_str = prompt_default(
        interactive,
        "Interval seconds",
        &crate::config::EffectiveConfig::default()
            .interval
            .to_string(),
    )?;
    let interval: u64 = interval_str
        .parse()
        .map_err(|e| anyhow!("invalid interval '{}': {}", interval_str, e))?;
    if interval == 0 {
        return Err(anyhow!("interval must be greater than zero"));
    }
    // The default log action is portable and safe.

    let cfg = Config {
        interval: Some(interval),
        isp_to_check: Some(isp),
        ..Config::default()
    };

    // Validate the generated config so the user sees failures immediately.
    let eff = cfg.merge_with_args(&crate::cli::Args::default());
    if let Err(errs) = crate::config::Config::validate_values(&eff) {
        let joined = errs.0.join("; ");
        return Err(anyhow!("generated config failed validation: {}", joined));
    }

    let default_path = {
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                std::path::Path::new(&home)
                    .join("Library")
                    .join("Preferences")
                    .join("check_vpn")
                    .join("config.xml")
                    .to_string_lossy()
                    .into_owned()
            } else {
                "./check_vpn.xml".to_string()
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            "./check_vpn.xml".to_string()
        }
    };

    let out_path: String = output
        .map(|s| s.to_string())
        .unwrap_or_else(|| default_path.clone());
    crate::xml_io::write_xml(&cfg, &out_path)
        .with_context(|| format!("failed to write generated config to {}", out_path))?;
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
    Ok(if v.is_empty() {
        default.to_string()
    } else {
        v.to_string()
    })
}

#[derive(Debug, thiserror::Error)]
#[error("initialization worker timed out or disconnected")]
pub struct InitTimeout;

// Bound the caller's wait; cancellation stops provider retries between requests.
pub fn run_with_timeout<F, R>(f: F, dur: std::time::Duration) -> Result<R, InitTimeout>
where
    F: FnOnce(Arc<AtomicBool>) -> R + Send + 'static,
    R: Send + 'static,
{
    use std::sync::mpsc;
    use std::thread;
    let cancel = Arc::new(AtomicBool::new(false));
    let cancel_for_thread = Arc::clone(&cancel);
    let (tx, rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        let _ = tx.send(f(cancel_for_thread));
    });

    match rx.recv_timeout(dur) {
        Ok(r) => {
            let _ = handle.join();
            Ok(r)
        }
        Err(_) => {
            cancel.store(true, Ordering::Relaxed);
            // Dropping the handle detaches the owned, cancellable worker.
            drop(handle);
            Err(InitTimeout)
        }
    }
}
