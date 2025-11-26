use anyhow::Result;
use reqwest::blocking::Client;
use std::time::Duration;

use crate::providers::{VpnIdentity, VpnInfoProvider};

pub struct IpApiProvider {
    client: Client,
    url: String,
    retries: usize,
}

impl IpApiProvider {
    pub fn new_default() -> anyhow::Result<Self> {
        let client = Client::builder()
            .user_agent("check_vpn/0.1")
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { client, url: "http://ip-api.com/json".to_string(), retries: 1 })
    }
}

impl VpnInfoProvider for IpApiProvider {
    fn name(&self) -> &str { "ip-api" }
    fn query(&self) -> Result<VpnIdentity> {
        let isp = crate::ip_api::get_isp_with_client_and_url(&self.client, &self.url, self.retries)?;
        Ok(VpnIdentity { isp })
    }
}
