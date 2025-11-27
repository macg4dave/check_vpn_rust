use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::RETRY_AFTER;
use serde::Deserialize;
use std::io::Read;
use std::thread::sleep;
use std::time::Duration;

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
            Ok(r) => {
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
                        // Exhausted retries — fall through to final unified error return
                        break;
                    }

                    // Retry on server errors
                    if status.is_server_error() {
                        last_err = Some(anyhow::anyhow!("non-success status: {}", status));
                        if attempt + 1 < retries {
                            sleep(Duration::from_millis(500 * (attempt as u64 + 1)));
                            continue;
                        }
                        // Exhausted retries — fall through to final unified error return
                        break;
                    }

                    return Err(anyhow::anyhow!("non-success status: {}", status));
                }

                // Enforce max content-length when provided
                match r.content_length() {
                    Some(len) if (len as usize) > max_bytes => {
                        return Err(anyhow::anyhow!("response too large: {} bytes", len));
                    }
                    _ => {}
                }

                // Read response body with a cap to avoid unbounded allocations
                let mut buf: Vec<u8> = Vec::new();
                let mut reader = r.take((max_bytes as u64) + 1);
                reader
                    .read_to_end(&mut buf)
                    .context("failed to read response body")?;
                if buf.len() > max_bytes {
                    return Err(anyhow::anyhow!("response too large (>{} bytes)", max_bytes));
                }

                let parsed: IpApiResponse =
                    serde_json::from_slice(&buf).context("failed to parse json")?;
                return parsed
                    .isp
                    .ok_or_else(|| anyhow::anyhow!("isp field missing in response"));
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

    // Return the last recorded error (with added context) if present, otherwise
    // return a generic error describing the failure to query the service.
    if let Some(e) = last_err {
        // Preserve the original error and add contextual information.
        Err(e.context(format!("failed to query ip api after {} attempt(s)", retries)))
    } else {
        Err(anyhow::anyhow!("failed to query ip api: no attempts performed"))
    }
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
    let url = std::env::var("CHECK_VPN_TEST_URL")
        .unwrap_or_else(|_| "http://ip-api.com/json".to_string());
    // Simple configurable retry policy via env var CHECK_VPN_RETRY_COUNT (default 1)
    let retries: usize = std::env::var("CHECK_VPN_RETRY_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

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
    use httpmock::Method::GET;
    use httpmock::MockServer;

    // A tiny RAII guard local to these tests to avoid leaving env vars set.
    struct TestScopedEnv {
        key: String,
        prev: Option<std::ffi::OsString>,
    }

    impl TestScopedEnv {
        fn set_str(key: &str, value: &str) -> TestScopedEnv {
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            TestScopedEnv {
                key: key.to_string(),
                prev,
            }
        }
    }

    impl Drop for TestScopedEnv {
        fn drop(&mut self) {
            if let Some(prev) = self.prev.take() {
                std::env::set_var(&self.key, prev);
            } else {
                std::env::remove_var(&self.key);
            }
        }
    }

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

        let _g = TestScopedEnv::set_str("CHECK_VPN_TEST_URL", &server.url("/json"));

        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        let isp =
            get_isp_with_client_and_url(&client, &server.url("/json"), 1).expect("get_isp failed");
        assert_eq!(isp, isp_value.to_string());

        mock.assert();
        drop(_g);
    }

    #[test]
    fn large_response_rejected_due_to_size_header() {
        let server = MockServer::start();
        let _g2 = TestScopedEnv::set_str("CHECK_VPN_MAX_RESPONSE_BYTES", "1024");

        let big_body = "x".repeat(2048);
        let _m = server.mock(|when, then| {
            when.method(GET).path("/json");
                then.status(200)
                .header("content-type", "application/json")
                .header("content-length", "2048")
                .body(big_body.as_str());
        });

        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();
        let res = get_isp_with_client_and_url(&client, &server.url("/json"), 1);
        drop(_g2);
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
        let call_count2 = std::sync::Arc::clone(&call_count);

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

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
        let res = get_isp_with_client_and_url(&client, &server.url("/json"), 2)
            .expect("expected success");
        assert_eq!(res, "X");
    }

    #[test]
    fn retries_exhausted_returns_error_with_context() {
        // Server always returns 500; with limited retries this should
        // cause get_isp_with_client_and_url to eventually return an error
        // containing useful context.
        let server = MockServer::start();
        let _m = server.mock(|when, then| {
            when.method(GET).path("/json");
            then.status(500).body("internal error");
        });

        let client = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap();

        let res = get_isp_with_client_and_url(&client, &server.url("/json"), 2);
        assert!(res.is_err(), "expected an error when retries are exhausted");
        let err = res.unwrap_err();
        let s = format!("{}", err);
        // Ensure our added context is present in the returned error
        assert!(s.contains("failed to query ip api"), "error missing context: {}", s);
        assert!(s.contains("2"), "error should mention attempts in: {}", s);
    }
}
