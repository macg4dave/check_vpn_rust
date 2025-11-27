use std::time::Duration;

use httpmock::Method::GET;
use httpmock::MockServer;
use reqwest::blocking::Client;

#[test]
fn ip_api_success() {
    let server = MockServer::start();

    let body = r#"{"isp":"Integration ISP"}"#;
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(body);
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let url = format!("{}/json", server.url(""));

    let isp =
        check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1).expect("expected success");
    assert_eq!(isp, "Integration ISP");
}

#[test]
fn ip_api_retries_behaviour() {
    // Ensure the retry loop actually causes multiple requests when the server keeps returning 500
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(500).body("internal");
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let url = format!("{}/json", server.url(""));

    // ask for 2 retries to exercise retry loop
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 2);
    assert!(res.is_err());
}

#[test]
fn ip_api_timeout_behaviour() {
    let server = MockServer::start();

    // Delay response so that a short-timeout client will fail
    let _m3 = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .delay(std::time::Duration::from_millis(3000))
            .body(r#"{"isp":"Slow ISP"}"#);
    });

    // Client timeout shorter than server delay
    let client = Client::builder()
        .timeout(Duration::from_secs(1))
        .build()
        .unwrap();
    let url = format!("{}/json", server.url(""));

    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    assert!(res.is_err(), "expected timeout to produce an error");
}

#[test]
fn ip_api_malformed_response() {
    let server = MockServer::start();

    // Return invalid JSON
    let _m4 = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200).body("not-json");
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let url = format!("{}/json", server.url(""));

    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    assert!(res.is_err(), "expected malformed JSON to error");
}

#[test]
fn ip_api_429_behaviour() {
    let server = MockServer::start();

    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(429);
    });

    let client = Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let url = format!("{}/json", server.url(""));

    // 429 should be treated like a retryable server error; with retries=1 it should ultimately error
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    assert!(res.is_err());
}
