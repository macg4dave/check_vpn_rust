use crate::{actions, config::EffectiveConfig, error_types::AppError};
use anyhow::Result as AnyhowResult;
use log::{error, info, warn};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckOutcome {
    VpnActive(String),
    RecoveryAttempted(String),
    RecoveryDeferred(String),
}

/// Query first: a blocked diagnostic endpoint must not hide a working identity source.
/// Errors are returned so the daemon can apply one consistent policy.
pub fn check_identity<FGet, FRun>(
    eff: &EffectiveConfig,
    action: &actions::Action,
    allow_recovery: bool,
    get_isp: FGet,
    run_action: FRun,
) -> Result<CheckOutcome, AppError>
where
    FGet: FnOnce() -> AnyhowResult<String>,
    FRun: FnOnce(&actions::Action, bool) -> AnyhowResult<()>,
{
    let isp = get_isp().map_err(AppError::RuntimeError)?;
    if isp.trim().is_empty() {
        return Err(AppError::RuntimeError(anyhow::anyhow!(
            "provider returned an empty ISP"
        )));
    }
    if !eff.matches_normal_isp(&isp) {
        info!("VPN active (ISP: {isp})");
        return Ok(CheckOutcome::VpnActive(isp));
    }
    warn!("VPN Lost (ISP: {isp})");
    if !allow_recovery {
        info!("Recovery deferred until the cooldown expires");
        return Ok(CheckOutcome::RecoveryDeferred(isp));
    }
    run_action(action, eff.dry_run).map_err(AppError::ActionFailed)?;
    Ok(CheckOutcome::RecoveryAttempted(isp))
}

pub(crate) fn handle_error(
    eff: &EffectiveConfig,
    result: Result<(), AppError>,
) -> Result<(), AppError> {
    match result {
        Err(error) if !eff.run_once && !eff.exit_on_error => {
            error!("VPN check failed; continuing monitoring: {error:#}");
            Ok(())
        }
        other => other,
    }
}

/// Compatibility helper for a single check with injected identity/action functions.
pub fn perform_check<FGet, FRun>(
    eff: &EffectiveConfig,
    get_isp: FGet,
    run_action: FRun,
) -> Result<(), AppError>
where
    FGet: FnOnce() -> AnyhowResult<String>,
    FRun: FnOnce(&actions::Action, bool) -> AnyhowResult<()>,
{
    let action = actions::parse_action(&eff.action_type, &eff.action_arg);
    handle_error(
        eff,
        check_identity(eff, &action, true, get_isp, run_action).map(|_| ()),
    )
}
