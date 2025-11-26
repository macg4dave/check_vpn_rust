use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;

use crate::providers::{VpnIdentity, VpnInfoProvider};

#[derive(Deserialize, Debug)]
struct IfconfigResponse {
    isp: Option<String>,           // Some mirrors might add this
    #[serde(rename = "asn_org")] 
    asn_org: Option<String>,
}

pub struct IfconfigCoProvider {
    client: Client,
    url: String,
}

impl IfconfigCoProvider {
    pub fn new_default() -> Result<Self> {
        let client = Client::builder()
            .user_agent("check_vpn/0.1")
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { client, url: "https://ifconfig.co/json".to_string() })
    }
}

impl VpnInfoProvider for IfconfigCoProvider {
    fn name(&self) -> &str { "ifconfig.co" }
    fn query(&self) -> Result<VpnIdentity> {
        let resp = self.client.get(&self.url).send().context("http request failed")?;
        if !resp.status().is_success() { return Err(anyhow::anyhow!("non-success status: {}", resp.status())); }
        let parsed: IfconfigResponse = resp.json().context("failed to parse ifconfig.co json")?;
        let isp = parsed.isp.or(parsed.asn_org).ok_or_else(|| anyhow::anyhow!("missing isp/asn_org field"))?;
        Ok(VpnIdentity { isp })
    }
}
