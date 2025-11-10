use anyhow::{Result, Context};
use reqwest::blocking::Client;
use serde_json::Value;
use std::time::Duration;

use crate::providers::{VpnIdentity, VpnInfoProvider};

pub struct GenericJsonProvider {
    client: Client,
    url: String,
    preferred_key: Option<String>,
}

impl GenericJsonProvider {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("check_vpn/0.1")
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self { client, url: url.to_string(), preferred_key: None })
    }

    pub fn with_key(mut self, key: Option<String>) -> Self {
        self.preferred_key = key;
        self
    }
}

impl VpnInfoProvider for GenericJsonProvider {
    fn name(&self) -> &str { &self.url }
    fn query(&self) -> Result<VpnIdentity> {
        let resp = self.client.get(&self.url).send().context("http request failed")?;
        if !resp.status().is_success() { return Err(anyhow::anyhow!("non-success status: {}", resp.status())); }
        let val: Value = resp.json().context("failed to parse json")?;
        // Preferred key from configuration first
        let isp = if let Some(ref k) = self.preferred_key {
            val.get(k).and_then(|v| v.as_str())
        } else { None };

        let isp = isp
            .or_else(|| val.get("isp").and_then(|v| v.as_str()))
            .or_else(|| val.get("asn_org").and_then(|v| v.as_str()))
            .or_else(|| val.get("org").and_then(|v| v.as_str()))
            .ok_or_else(|| anyhow::anyhow!("no recognizable field (tried preferred, isp, asn_org, org)"))?;
        Ok(VpnIdentity { isp: isp.to_string() })
    }
}
