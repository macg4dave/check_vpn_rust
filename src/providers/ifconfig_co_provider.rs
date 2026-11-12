use super::{generic_json_provider::GenericJsonProvider, VpnIdentity, VpnInfoProvider};
use anyhow::Result;
use reqwest::blocking::Client;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
pub const DEFAULT_URL: &str = "https://ifconfig.co/json";
pub struct IfconfigCoProvider(GenericJsonProvider);
impl IfconfigCoProvider {
    pub fn new_default() -> Result<Self> {
        Self::new_with_url(DEFAULT_URL)
    }
    pub fn new_with_url(url: &str) -> Result<Self> {
        Ok(Self(GenericJsonProvider::new(url)?))
    }
    pub fn with_client(client: Client) -> Result<Self> {
        Ok(Self(GenericJsonProvider::with_client(DEFAULT_URL, client)?))
    }
}
impl VpnInfoProvider for IfconfigCoProvider {
    fn name(&self) -> &str {
        "ifconfig.co"
    }
    fn query(&self) -> Result<VpnIdentity> {
        self.0.query()
    }
    fn query_with_cancel(
        &self,
        cancel: Option<&AtomicBool>,
        budget: Duration,
    ) -> Result<VpnIdentity> {
        self.0.query_with_cancel(cancel, budget)
    }
}
