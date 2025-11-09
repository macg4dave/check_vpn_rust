use httpmock::Method::GET;
use httpmock::MockServer;

#[test]
fn get_isp_success_with_mock() {
    // Start a local mock server (synchronous, suitable for blocking reqwest client)
    let server = MockServer::start();

    // create mock that returns JSON with isp
    let isp_body = r#"{"isp":"Test ISP Co"}"#;
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(isp_body);
    });

    // Build a client with the same defaults the library uses
    let client = reqwest::blocking::Client::builder()
        .user_agent("check_vpn/0.1")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("failed to build client");

    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &server.url("/json"), 1);
    assert!(res.is_ok());
    let isp = res.unwrap();
    assert_eq!(isp, "Test ISP Co");
}

#[test]
fn get_isp_server_error_retries_then_error() {
    let server = MockServer::start();

    // return 500
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(500).body("internal");
    });

    let client = reqwest::blocking::Client::builder()
        .user_agent("check_vpn/0.1")
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("failed to build client");

    // ask for 2 retries to exercise retry loop
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &server.url("/json"), 2);
    assert!(res.is_err());
}
