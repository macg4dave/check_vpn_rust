//! Ordered provider fallback with shared HTTP transport and a total budget.
use anyhow::{bail, Result};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};
pub mod generic_json_provider;
pub mod http;
pub mod ifconfig_co_provider;
pub mod ip_api_provider;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpnIdentity {
    pub isp: String,
}
pub trait VpnInfoProvider: Send + Sync {
    fn name(&self) -> &str;
    fn query(&self) -> Result<VpnIdentity>;
    fn query_with_cancel(
        &self,
        _cancel: Option<&AtomicBool>,
        _budget: Duration,
    ) -> Result<VpnIdentity> {
        self.query()
    }
}
pub fn build_chain(eff: &crate::config::EffectiveConfig) -> Result<Vec<Box<dyn VpnInfoProvider>>> {
    let client = http::client()?;
    let mut chain: Vec<Box<dyn VpnInfoProvider>> = Vec::new();
    for spec in eff.provider_specs() {
        chain.push(Box::new(
            generic_json_provider::GenericJsonProvider::with_client(&spec.url, client.clone())?
                .with_key(spec.key.as_deref()),
        ));
    }
    if eff.enable_ip_api {
        chain.push(Box::new(ip_api_provider::IpApiProvider::with_client(
            client.clone(),
        )?));
    }
    if eff.enable_ifconfig_co {
        chain.push(Box::new(
            ifconfig_co_provider::IfconfigCoProvider::with_client(client)?,
        ));
    }
    if chain.is_empty() {
        bail!("no providers configured");
    }
    Ok(chain)
}
pub fn query_first_success(providers: &[Box<dyn VpnInfoProvider>]) -> Result<VpnIdentity> {
    query(providers, None)
}
pub fn query_first_success_with_cancel(
    providers: &[Box<dyn VpnInfoProvider>],
    cancel: &AtomicBool,
) -> Result<VpnIdentity> {
    query(providers, Some(cancel))
}
fn query(
    providers: &[Box<dyn VpnInfoProvider>],
    cancel: Option<&AtomicBool>,
) -> Result<VpnIdentity> {
    let start = Instant::now();
    let mut errors = Vec::new();
    for provider in providers {
        if http::cancelled(cancel) {
            bail!("provider query cancelled");
        }
        let remaining = http::QUERY_BUDGET.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            errors.push("provider query deadline exceeded".into());
            break;
        }
        match provider.query_with_cancel(cancel, remaining.min(Duration::from_secs(10))) {
            Ok(id) if !id.isp.trim().is_empty() => return Ok(id),
            Ok(_) => errors.push(format!("{}: empty ISP identity", provider.name())),
            Err(error) => errors.push(format!("{}: {error:#}", provider.name())),
        }
    }
    if errors.is_empty() {
        bail!("no providers configured");
    }
    bail!("provider lookup failed: {}", errors.join("; "))
}
