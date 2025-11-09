use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    isp: Option<String>,
}

/// Query ip-api.com for the current public ISP.
/// Returns the ISP string on success or an error.
pub fn get_isp() -> Result<String> {
    // Allow tests to override the endpoint via environment variable.
    let url = std::env::var("CHECK_VPN_TEST_URL").unwrap_or_else(|_| "http://ip-api.com/json".to_string());
    let client = reqwest::blocking::Client::builder()
        .user_agent("check_vpn/0.1")
        .build()
        .context("failed to build http client")?;

    let resp = client
        .get(url)
        .send()
        .context("http request failed")?;

    if !resp.status().is_success() {
        anyhow::bail!("non-success status: {}", resp.status());
    }

    let parsed: IpApiResponse = resp.json().context("failed to parse json")?;

    match parsed.isp {
        Some(s) => Ok(s),
        None => anyhow::bail!("isp field missing in response"),
    }
}
