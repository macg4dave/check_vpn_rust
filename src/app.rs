use crate::actions::{self, runner::RealActionRunner, PreparedAction};
use crate::cli::Args;
use crate::config::{Config, ConfigSource, EffectiveConfig};
use crate::error_types::AppError;
use crate::providers::{self, VpnInfoProvider};
use log::{debug, info, warn};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod check;
pub use check::{check_identity, perform_check, CheckOutcome};

struct PreparedConfig {
    eff: EffectiveConfig,
    action: PreparedAction,
    providers: Vec<Box<dyn VpnInfoProvider>>,
}
impl PreparedConfig {
    fn new(eff: EffectiveConfig) -> Result<Self, AppError> {
        Config::validate_values(&eff)?;
        let action = PreparedAction::new(actions::parse_action(&eff.action_type, &eff.action_arg))
            .map_err(AppError::ConfigLoadError)?;
        let providers = providers::build_chain(&eff).map_err(AppError::ConfigLoadError)?;
        Ok(Self {
            eff,
            action,
            providers,
        })
    }
}

/// Library entry point. The binary passes its already-resolved source instead.
pub fn run(args: Args, cfg: Config) -> Result<(), AppError> {
    let source =
        ConfigSource::resolve(args.config.as_deref()).map_err(AppError::ConfigLoadError)?;
    run_with_source(args, cfg, source)
}

fn reload(
    args: &Args,
    source: &ConfigSource,
    current: &mut Config,
    prepared: &mut PreparedConfig,
) -> Result<(), AppError> {
    if matches!(source, ConfigSource::Defaults) {
        return Ok(());
    }
    let next = source.load().map_err(AppError::ConfigLoadError)?;
    if next != *current {
        let eff = next.merge_with_args(args);
        // Timing/action-only edits reuse the existing connections.
        if eff == prepared.eff {
            *current = next;
            return Ok(());
        }
        Config::validate_values(&eff)?;
        let action = PreparedAction::new(actions::parse_action(&eff.action_type, &eff.action_arg))
            .map_err(AppError::ConfigLoadError)?;
        let providers_changed = eff.provider_specs() != prepared.eff.provider_specs()
            || eff.enable_ip_api != prepared.eff.enable_ip_api
            || eff.enable_ifconfig_co != prepared.eff.enable_ifconfig_co;
        let replacement = if providers_changed {
            Some(providers::build_chain(&eff).map_err(AppError::ConfigLoadError)?)
        } else {
            None
        };
        // Commit only once all replacements are ready.
        if let Some(chain) = replacement {
            prepared.providers = chain;
        }
        prepared.eff = eff;
        prepared.action = action;
        *current = next;
        debug!("Applied reloaded configuration");
    }
    Ok(())
}

pub(crate) fn run_with_source(
    args: Args,
    mut cfg: Config,
    source: ConfigSource,
) -> Result<(), AppError> {
    info!("Starting check_vpn run()");
    let mut prepared = PreparedConfig::new(cfg.merge_with_args(&args))?;
    let cancel = Arc::new(AtomicBool::new(false));
    if !prepared.eff.run_once {
        let signal_cancel = Arc::clone(&cancel);
        ctrlc::set_handler(move || {
            signal_cancel.store(true, Ordering::Relaxed);
        })
        .map_err(|e| AppError::RuntimeError(e.into()))?;
    }
    let runner = RealActionRunner::new();
    let last_attempt = Cell::new(None::<Instant>);
    while !cancel.load(Ordering::Relaxed) {
        if !prepared.eff.run_once {
            if let Err(error) = reload(&args, &source, &mut cfg, &mut prepared) {
                if args.strict_config {
                    return Err(AppError::ConfigLoadError(
                        anyhow::Error::new(error).context("--strict-config is set"),
                    ));
                }
                warn!("Config reload failed; keeping previous configuration: {error:#}");
            }
        }
        let allowed = last_attempt.get().is_none_or(|last| {
            last.elapsed() >= Duration::from_secs(prepared.eff.recovery_cooldown_secs)
        });
        let result = check_identity(
            &prepared.eff,
            &prepared.action.action,
            allowed,
            || {
                providers::query_first_success_with_cancel(&prepared.providers, &cancel)
                    .map(|id| id.isp)
            },
            |_, dry_run| {
                // Charge failed attempts too, so a broken recovery command cannot spin.
                let result = runner.execute_prepared(
                    &prepared.action,
                    dry_run,
                    Duration::from_secs(prepared.eff.action_timeout_secs),
                    &cancel,
                );
                last_attempt.set(Some(Instant::now()));
                result
            },
        );
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        let result = result.map(|_| ());
        // TCP probes are diagnostics only and never gate identity lookup/recovery.
        if matches!(result, Err(AppError::RuntimeError(_))) {
            let online = crate::networking::is_online_with_cancel(
                &prepared.eff.connectivity_endpoints,
                prepared.eff.connectivity_timeout_secs,
                &prepared.eff.connectivity_ports,
                prepared.eff.connectivity_retries,
                &cancel,
                Duration::from_secs(5),
            );
            debug!("Connectivity diagnostics after lookup failure: {online:?}");
        }
        if cancel.load(Ordering::Relaxed) {
            break;
        }
        check::handle_error(&prepared.eff, result)?;
        if prepared.eff.run_once {
            return Ok(());
        }
        // The interval remains a delay after each completed check.
        let _ = providers::http::wait(Duration::from_secs(prepared.eff.interval), Some(&cancel));
    }
    info!("Exiting check_vpn run loop");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejected_reloads_leave_all_runtime_state_untouched() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.xml");
        let source = ConfigSource::File(path.clone());
        let args = Args::default();
        let mut current = Config::default();
        let mut prepared = PreparedConfig::new(current.merge_with_args(&args)).unwrap();
        let before = prepared.eff.clone();
        let clients = prepared.providers.as_ptr();
        for xml in [
            "<bad>",
            "<config><interval>0</interval></config>",
            "<config><provider_urls>bad URL</provider_urls></config>",
        ] {
            std::fs::write(&path, xml).unwrap();
            assert!(reload(&args, &source, &mut current, &mut prepared).is_err());
            assert_eq!(prepared.eff, before);
            assert_eq!(prepared.providers.as_ptr(), clients);
        }
        std::fs::remove_file(&path).unwrap();
        assert!(reload(&args, &source, &mut current, &mut prepared).is_err());
        assert_eq!(prepared.eff, before);
    }

    #[test]
    fn timing_reload_reuses_clients_and_cli_overrides_survive() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.xml");
        let source = ConfigSource::File(path.clone());
        let args = Args {
            isp_to_check: Some("CLI normal ISP".into()),
            ..Default::default()
        };
        let mut current = Config::default();
        let mut prepared = PreparedConfig::new(current.merge_with_args(&args)).unwrap();
        let clients = prepared.providers.as_ptr();
        std::fs::write(
            path,
            "<config><interval>7</interval><isp_to_check>File ISP</isp_to_check></config>",
        )
        .unwrap();
        reload(&args, &source, &mut current, &mut prepared).unwrap();
        assert_eq!(prepared.eff.interval, 7);
        assert_eq!(prepared.eff.isp_to_check, "CLI normal ISP");
        assert_eq!(prepared.providers.as_ptr(), clients);
    }
}
