use httpmock::Method::GET;
use httpmock::MockServer;
use std::env;
use std::time::Duration;

#[test]
fn large_response_rejected_due_to_size_header() {
    let server = MockServer::start();

    // Configure small max via env so test runs fast and deterministically
    // Some targets treat env var mutation as unsafe; wrap in unsafe for portability.
    unsafe {
        env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", "1024");
    }

    // Return a Content-Length larger than the allowed 1024 bytes
    let big_body = "x".repeat(2048);
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .header("content-length", "2048")
            .body(big_body.clone());
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let url = server.url("/json");

    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    unsafe {
        env::remove_var("CHECK_VPN_MAX_RESPONSE_BYTES");
    }
    assert!(res.is_err(), "expected large response to be rejected");
}
