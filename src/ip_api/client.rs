use anyhow::Result;
use reqwest::blocking::Client;
pub const DEFAULT_IP_API_URL: &str = "https://ip-api.com/json";

pub fn get_isp_with_client_and_url(client: &Client, url: &str, retries: usize) -> Result<String> {
    let value = crate::providers::http::get_json(
        client,
        url,
        retries,
        None,
        std::time::Duration::from_secs(65),
    )?;
    crate::providers::http::identity(&value, &["isp"])
}
pub fn get_isp() -> Result<String> {
    let url = std::env::var("CHECK_VPN_TEST_URL").unwrap_or_else(|_| DEFAULT_IP_API_URL.into());
    let retries = std::env::var("CHECK_VPN_RETRY_COUNT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    get_isp_with_client_and_url(&crate::providers::http::client()?, &url, retries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;
    use std::time::Duration;

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
        assert!(
            s.contains("failed to query ip api"),
            "error missing context: {}",
            s
        );
        assert!(s.contains("2"), "error should mention attempts in: {}", s);
    }

    #[test]
    fn default_url_uses_https() {
        assert!(
            DEFAULT_IP_API_URL.starts_with("https://"),
            "default ip-api URL should use HTTPS"
        );
    }
}
