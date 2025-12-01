use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::io::Read;
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
        Ok(Self {
            client,
            url: url.to_string(),
            preferred_key: None,
        })
    }

    /// Set an optional preferred key to look up in returned JSON objects.
    ///
    /// Accepts an Option<&str> to avoid forcing callers to provide owned Strings
    /// when a borrowed &str is available; the provider will clone into an owned
    /// String internally only when necessary.
    pub fn with_key(mut self, key: Option<&str>) -> Self {
        self.preferred_key = key.map(|s| s.to_string());
        self
    }
}

impl VpnInfoProvider for GenericJsonProvider {
    fn name(&self) -> &str {
        &self.url
    }
    fn query(&self) -> Result<VpnIdentity> {
        // Reuse the same env override as ip-api for maximum response size.
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

        let val: Value = serde_json::from_slice(&buf).context("failed to parse json")?;
        // Preferred key from configuration first
        let isp = if let Some(ref k) = self.preferred_key {
            val.get(k).and_then(|v| v.as_str())
        } else {
            None
        };

        let isp = isp
            .or_else(|| val.get("isp").and_then(|v| v.as_str()))
            .or_else(|| val.get("asn_org").and_then(|v| v.as_str()))
            .or_else(|| val.get("org").and_then(|v| v.as_str()))
            .ok_or_else(|| {
                anyhow::anyhow!("no recognizable field (tried preferred, isp, asn_org, org)")
            })?;
        Ok(VpnIdentity {
            isp: isp.to_string(),
        })
    }
}
