use super::{http, VpnIdentity, VpnInfoProvider};
use anyhow::Result;
use reqwest::blocking::Client;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
pub struct GenericJsonProvider {
    client: Client,
    url: String,
    preferred_key: Option<String>,
}
impl GenericJsonProvider {
    pub fn new(url: &str) -> Result<Self> {
        Self::with_client(url, http::client()?)
    }
    pub fn with_client(url: &str, client: Client) -> Result<Self> {
        http::validate_url(url)?;
        Ok(Self {
            client,
            url: url.into(),
            preferred_key: None,
        })
    }
    pub fn with_key(mut self, key: Option<&str>) -> Self {
        self.preferred_key = key.map(str::to_owned);
        self
    }
}
impl VpnInfoProvider for GenericJsonProvider {
    fn name(&self) -> &str {
        &self.url
    }
    fn query(&self) -> Result<VpnIdentity> {
        self.query_with_cancel(None, http::QUERY_BUDGET)
    }
    fn query_with_cancel(
        &self,
        cancel: Option<&AtomicBool>,
        budget: Duration,
    ) -> Result<VpnIdentity> {
        let value = http::get_json(&self.client, &self.url, 2, cancel, budget)?;
        let mut keys = Vec::new();
        if let Some(key) = self.preferred_key.as_deref() {
            keys.push(key);
        }
        keys.extend(["isp", "asn_org", "org"]);
        Ok(VpnIdentity {
            isp: http::identity(&value, &keys)?,
        })
    }
}
