use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::RETRY_AFTER;
use serde::Deserialize;
use std::io::Read;
use std::time::Duration;
use std::thread::sleep;

/// Default maximum response body size in bytes before we reject the response.
const DEFAULT_MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024; // 5MB
/// Maximum Retry-After seconds to respect (clamp large values)
const MAX_RETRY_AFTER_SECS: u64 = 60;

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    isp: Option<String>,
}

/// Query ip-api.com for the current public ISP using a provided blocking HTTP client.
///
/// This function is test-friendly because callers can inject a client and URL.
pub fn get_isp_with_client_and_url(client: &Client, url: &str, retries: usize) -> Result<String> {
    let retries = std::cmp::max(1, retries);

    // Allow tests to override max response bytes via env var
    let max_bytes: usize = std::env::var("CHECK_VPN_MAX_RESPONSE_BYTES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_RESPONSE_BYTES);

    let mut last_err: Option<anyhow::Error> = None;

    for attempt in 0..retries {
        let resp = client.get(url).send();

        match resp {
            Ok(mut r) => {
                let status = r.status();
                // debug prints removed; keep logic compact
                if !status.is_success() {
                    // Special handling for 429 Too Many Requests where Retry-After may help
                    if status.as_u16() == 429 {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", status));
                        if attempt + 1 < retries {
                            if let Some(secs) = parse_retry_after_secs(&r) {
                                let secs = std::cmp::min(secs, MAX_RETRY_AFTER_SECS);
                                sleep(Duration::from_secs(secs));
                                continue;
                            }
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        return Err(anyhow::anyhow!("non-success status: {}", status));
                    }

                    // Retry on server errors
                    if status.is_server_error() {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", status));
                        if attempt + 1 < retries {
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        return Err(anyhow::anyhow!("non-success status: {}", status));
                    }

                    return Err(anyhow::anyhow!("non-success status: {}", status));
                }

                // Enforce max content-length when provided
                if let Some(len) = r.content_length() {
                    if (len as usize) > max_bytes {
                        return Err(anyhow::anyhow!("response too large: {} bytes", len));
                    }
                }

                // Read response body with a cap to avoid unbounded allocations
                let mut buf: Vec<u8> = Vec::new();
                let mut reader = r.take((max_bytes as u64) + 1);
                reader.read_to_end(&mut buf).context("failed to read response body")?;
                if buf.len() > max_bytes {
                    return Err(anyhow::anyhow!("response too large (>{} bytes)", max_bytes));
                }

                let parsed: IpApiResponse = serde_json::from_slice(&buf).context("failed to parse json")?;
                return parsed.isp.ok_or_else(|| anyhow::anyhow!("isp field missing in response"));
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

fn parse_retry_after_secs(resp: &reqwest::blocking::Response) -> Option<u64> {
    resp.headers()
        .get(RETRY_AFTER)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
}

/// Backwards-compatible helper: build a client and use env vars like before.
pub fn get_isp() -> Result<String> {
    // Allow tests to override the endpoint via environment variable.
    let url = std::env::var("CHECK_VPN_TEST_URL").unwrap_or_else(|_| "http://ip-api.com/json".to_string());
    // Simple configurable retry policy via env var CHECK_VPN_RETRY_COUNT (default 1)
    let retries: usize = std::env::var("CHECK_VPN_RETRY_COUNT").ok().and_then(|s| s.parse().ok()).unwrap_or(1);

    let client = Client::builder()
        .user_agent("check_vpn/0.1")
        .timeout(Duration::from_secs(5))
        .build()
        .context("failed to build http client")?;

    get_isp_with_client_and_url(&client, &url, retries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::MockServer;
    use httpmock::Method::GET;

    #[test]
    fn get_isp_parses_isp_field() {
        let server = MockServer::start();
        let isp_value = "Hutchison 3G UK Ltd";
        let mock = server.mock(|when, then| {
            when.method(GET).path("/json");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!("{{ \"isp\": \"{}\" }}", isp_value));
        });

        unsafe { std::env::set_var("CHECK_VPN_TEST_URL", server.url("/json")); }

        let client = Client::builder().timeout(Duration::from_secs(2)).build().unwrap();
        let isp = get_isp_with_client_and_url(&client, &server.url("/json"), 1).expect("get_isp failed");
        assert_eq!(isp, isp_value.to_string());

        mock.assert();
        unsafe { std::env::remove_var("CHECK_VPN_TEST_URL"); }
    }

    #[test]
    fn large_response_rejected_due_to_size_header() {
        let server = MockServer::start();
        unsafe { std::env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", "1024"); }

        let big_body = "x".repeat(2048);
        let _m = server.mock(|when, then| {
            when.method(GET).path("/json");
            then.status(200)
                .header("content-type", "application/json")
                .header("content-length", "2048")
                .body(big_body.clone());
        });

        let client = Client::builder().timeout(Duration::from_secs(2)).build().unwrap();
        let res = get_isp_with_client_and_url(&client, &server.url("/json"), 1);
        unsafe { std::env::remove_var("CHECK_VPN_MAX_RESPONSE_BYTES"); }
        assert!(res.is_err(), "expected large response to be rejected");
    }

    #[test]
    fn retry_after_header_respected() {
        // This test ensures that when the server responds 429 with Retry-After
        // header, the client waits and retries (we simulate with two mocks).
        let server = MockServer::start();
        use httpmock::HttpMockResponse;
        use std::sync::Mutex;

        // Use a responding closure to simulate a 429 on the first call and 200 on the second.
        let call_count = std::sync::Arc::new(Mutex::new(0));
        let call_count2 = call_count.clone();

        let _m = server.mock(move |when, then| {
            when.method(GET).path("/json");
            then.respond_with(move |_req| {
                let mut count = call_count2.lock().unwrap();
                *count += 1;
                if *count == 1 {
                    HttpMockResponse::builder()
                        .status(429)
                        .header("Retry-After", "1")
                        .body("")
                        .build()
                } else {
                    HttpMockResponse::builder()
                        .status(200)
                        .body("{ \"isp\": \"X\" }")
                        .build()
                }
            });
        });

        let client = Client::builder().timeout(Duration::from_secs(5)).build().unwrap();
        let res = get_isp_with_client_and_url(&client, &server.url("/json"), 2).expect("expected success");
        assert_eq!(res, "X");
    }
}
