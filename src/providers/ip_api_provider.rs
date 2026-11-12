use super::{http, VpnIdentity, VpnInfoProvider};
use crate::ip_api::client::DEFAULT_IP_API_URL;
use anyhow::Result;
use reqwest::blocking::Client;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
pub struct IpApiProvider {
    client: Client,
    url: String,
}
impl IpApiProvider {
    pub fn new_default() -> Result<Self> {
        Self::with_client(http::client()?)
    }
    pub fn with_client(client: Client) -> Result<Self> {
        Ok(Self {
            client,
            url: DEFAULT_IP_API_URL.into(),
        })
    }
    pub fn endpoint(&self) -> &str {
        &self.url
    }
}
impl VpnInfoProvider for IpApiProvider {
    fn name(&self) -> &str {
        "ip-api"
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
        Ok(VpnIdentity {
            isp: http::identity(&value, &["isp"])?,
        })
    }
}
