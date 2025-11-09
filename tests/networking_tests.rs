use httpmock::MockServer;
use httpmock::Method::GET;

#[test]
fn get_isp_parses_isp_field() {
    // Start a local mock server
    let server = MockServer::start();

    // Prepare a mock response matching the ip-api.com JSON shape (we only need isp)
    let isp_value = "Hutchison 3G UK Ltd";
    let mock = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .body(format!("{{ \"isp\": \"{}\" }}", isp_value));
    });

    // Point ip_api to the mock server via env var
    // On some targets `std::env::set_var` is considered unsafe; wrap in an
    // `unsafe` block to satisfy those targets.
    unsafe { std::env::set_var("CHECK_VPN_TEST_URL", server.url("/json")); }

    // Call the function under test
    let isp = check_vpn::ip_api::get_isp().expect("get_isp failed");
    assert_eq!(isp, isp_value.to_string());

    mock.assert();
}

