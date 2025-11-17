use reqwest::blocking::Client;

// These tests use real network calls and are ignored by default. Run with
// `cargo test -- --ignored` to execute them on a machine with network access.

#[test]
#[ignore]
fn real_ip_api_returns_200_and_parses() {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let url = "http://ip-api.com/json".to_string();
    let isp = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1)
        .expect("expected real ip-api to succeed");
    assert!(
        !isp.trim().is_empty(),
        "expected non-empty ISP from real ip-api"
    );
}

#[test]
#[ignore]
fn real_timeout_behaviour() {
    // This test aims to verify timeout behaviour against a slow host; it's
    // marked ignored because it depends on remote conditions.
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(1))
        .build()
        .unwrap();
    // Use a host that likely responds slowly (example.com is usually fast, so
    // this test is best run when you know a target); kept as placeholder.
    let url = "http://httpbin.org/delay/3".to_string();
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    assert!(
        res.is_err(),
        "expected timeout to produce an error against slow endpoint"
    );
}
