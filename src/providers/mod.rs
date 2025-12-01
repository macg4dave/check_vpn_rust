//! Provider abstraction layer for obtaining ISP / network identity information.
//!
//! This module introduces a `VpnInfoProvider` trait allowing multiple upstream
//! services to be queried in sequence (fallback). Existing ip-api logic is
//! adapted via a thin wrapper; new providers (e.g. ifconfig.co) implement the
//! same interface. Custom JSON endpoints can also be configured via CLI / XML.
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};

/// Resulting minimal info we care about for VPN loss detection. Extendable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VpnIdentity {
    pub isp: String,
}

/// Trait implemented by all provider clients.
pub trait VpnInfoProvider: Send + Sync {
    fn name(&self) -> &str;
    fn query(&self) -> Result<VpnIdentity>;
}

/// Helper to iterate through providers in order and return the first success.
pub fn query_first_success(providers: &[Box<dyn VpnInfoProvider>]) -> Result<VpnIdentity> {
    let mut last_err: Option<anyhow::Error> = None;
    for p in providers {
        match p.query() {
            Ok(id) => return Ok(id),
            Err(e) => {
                last_err = Some(anyhow::anyhow!("{}: {}", p.name(), e));
                continue;
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no providers configured")))
}

/// Variant of `query_first_success` that bails out early when `cancel` is set.
pub fn query_first_success_with_cancel(
    providers: &[Box<dyn VpnInfoProvider>],
    cancel: &AtomicBool,
) -> Result<VpnIdentity> {
    let mut last_err: Option<anyhow::Error> = None;
    for p in providers {
        if cancel.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("provider query cancelled"));
        }
        match p.query() {
            Ok(id) => return Ok(id),
            Err(e) => {
                last_err = Some(anyhow::anyhow!("{}: {}", p.name(), e));
                continue;
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no providers configured")))
}

pub mod generic_json_provider;
pub mod ifconfig_co_provider;
pub mod ip_api_provider;
