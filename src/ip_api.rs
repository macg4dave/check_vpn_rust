use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use std::thread::sleep;

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    isp: Option<String>,
}

/// Query ip-api.com for the current public ISP.
/// Returns the ISP string on success or an error.
/// Query ip-api.com (or a provided URL) for the current public ISP using
/// a provided blocking HTTP client. This is testable because the caller
/// can inject the client, URL and retry policy.
pub fn get_isp_with_client_and_url(
    client: &reqwest::blocking::Client,
    url: &str,
    mut retries: usize,
) -> Result<String> {
    if retries == 0 {
        retries = 1;
    }

    let mut last_err: Option<anyhow::Error> = None;
    for attempt in 0..retries {
        let resp = client.get(url).send();

        match resp {
            Ok(r) => {
                if !r.status().is_success() {
                    // Retry on server errors and 429; for other client errors bail out.
                    if r.status().is_server_error() || r.status().as_u16() == 429 {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", r.status()));
                        // small backoff before retrying
                        if attempt + 1 < retries {
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        return Err(anyhow::anyhow!("non-success status: {}", r.status()));
                    } else {
                        return Err(anyhow::anyhow!("non-success status: {}", r.status()));
                    }
                }

                let parsed: IpApiResponse = r.json().context("failed to parse json")?;
                return match parsed.isp {
                    Some(s) => Ok(s),
                    None => Err(anyhow::anyhow!("isp field missing in response")),
                };
            }
            Err(e) => {
                last_err = Some(anyhow::anyhow!("http request failed: {}", e));
                if attempt + 1 < retries {
                    sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                    continue;
                }
            }
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("failed to query ip api")))
}

/// Backwards-compatible helper: build a client and use env vars like before.
pub fn get_isp() -> Result<String> {
    // Allow tests to override the endpoint via environment variable.
    let url = std::env::var("CHECK_VPN_TEST_URL").unwrap_or_else(|_| "http://ip-api.com/json".to_string());
    // Simple configurable retry policy via env var CHECK_VPN_RETRY_COUNT (default 1)
    let retries: usize = std::env::var("CHECK_VPN_RETRY_COUNT").ok().and_then(|s| s.parse().ok()).unwrap_or(1);

    let client = reqwest::blocking::Client::builder()
        .user_agent("check_vpn/0.1")
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build http client")?;

    get_isp_with_client_and_url(&client, &url, retries)
}
