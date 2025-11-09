use anyhow::{Context, Result};
use serde::Deserialize;
use std::time::Duration;
use std::thread::sleep;
use std::io::Read;
use reqwest::header::RETRY_AFTER;

/// Default maximum response body size in bytes before we reject the response.
const DEFAULT_MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024; // 5MB
/// Maximum Retry-After seconds to respect (clamp large values)
const MAX_RETRY_AFTER_SECS: u64 = 60;

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
    // Allow tests to override max response bytes via env var
    let max_bytes: usize = std::env::var("CHECK_VPN_MAX_RESPONSE_BYTES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES);
    for attempt in 0..retries {
    let resp = client.get(url).send();

        match resp {
            Ok(mut r) => {
                if !r.status().is_success() {
                    // Handle 429 (Too Many Requests) specially if Retry-After header present
                    if r.status().as_u16() == 429 {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", r.status()));
                        if attempt + 1 < retries {
                            // Try to respect Retry-After header if present (numeric seconds)
                            if let Some(ra) = r.headers().get(RETRY_AFTER) {
                                if let Ok(s) = ra.to_str() {
                                    if let Ok(secs) = s.parse::<u64>() {
                                        let secs = std::cmp::min(secs, MAX_RETRY_AFTER_SECS);
                                        sleep(Duration::from_secs(secs));
                                        continue;
                                    }
                                }
                            }
                            // fallback linear backoff
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        return Err(anyhow::anyhow!("non-success status: {}", r.status()));
                    }

                    // Retry on server errors; other client errors bail out immediately.
                    if r.status().is_server_error() {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", r.status()));
                        if attempt + 1 < retries {
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        return Err(anyhow::anyhow!("non-success status: {}", r.status()));
                    } else {
                        return Err(anyhow::anyhow!("non-success status: {}", r.status()));
                    }
                }

                // Read response body with an enforced maximum size to avoid OOM on large responses.
                if let Some(len) = r.content_length() {
                    if len > max_bytes as u64 {
                        return Err(anyhow::anyhow!("response too large: {} bytes", len));
                    }
                }

                let mut buf: Vec<u8> = Vec::new();
                // r implements Read for blocking client; use take to cap bytes read
                let mut reader = r.take((max_bytes as u64) + 1);
                reader
                    .read_to_end(&mut buf)
                    .context("failed to read response body")?;
                if buf.len() > max_bytes {
                    return Err(anyhow::anyhow!("response too large (>{} bytes)", max_bytes));
                }

                let parsed: IpApiResponse = serde_json::from_slice(&buf).context("failed to parse json")?;
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
