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
    let url = "https://ip-api.com/json".to_string();
    let isp = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1)
        .expect("expected real ip-api to succeed");
    assert!(
        !isp.trim().is_empty(),
        "expected non-empty ISP from real ip-api"
    );
}
