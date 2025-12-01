use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::io::Read;
use std::time::Duration;

use crate::providers::{VpnIdentity, VpnInfoProvider};

#[derive(Deserialize, Debug)]
struct IfconfigResponse {
    isp: Option<String>, // Some mirrors might add this
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
        Ok(Self {
            client,
            url: "https://ifconfig.co/json".to_string(),
        })
    }

    pub fn new_with_url(url: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("check_vpn/0.1")
            .timeout(Duration::from_secs(5))
            .build()?;
        Ok(Self {
            client,
            url: url.to_string(),
        })
    }
}

impl VpnInfoProvider for IfconfigCoProvider {
    fn name(&self) -> &str {
        "ifconfig.co"
    }
    fn query(&self) -> Result<VpnIdentity> {
        let max_bytes: usize = std::env::var("CHECK_VPN_MAX_RESPONSE_BYTES")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(5 * 1024 * 1024);

        let resp = self
            .client
            .get(&self.url)
            .send()
            .context("http request failed")?;
        if !resp.status().is_success() {
            return Err(anyhow::anyhow!("non-success status: {}", resp.status()));
        }

        if let Some(len) = resp.content_length() {
            if (len as usize) > max_bytes {
                return Err(anyhow::anyhow!("response too large: {} bytes", len));
            }
        }

        let mut buf = Vec::new();
        let mut reader = resp.take((max_bytes as u64) + 1);
        reader
            .read_to_end(&mut buf)
            .context("failed to read response body")?;
        if buf.len() > max_bytes {
            return Err(anyhow::anyhow!("response too large (>{} bytes)", max_bytes));
        }

        let parsed: IfconfigResponse =
            serde_json::from_slice(&buf).context("failed to parse ifconfig.co json")?;
        let isp = parsed
            .isp
            .or(parsed.asn_org)
            .ok_or_else(|| anyhow::anyhow!("missing isp/asn_org field"))?;
        Ok(VpnIdentity { isp })
    }
}
