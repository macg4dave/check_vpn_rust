use httpmock::Method::GET;
use httpmock::MockServer;
mod support;
use serial_test::serial;

#[test]
#[serial(env)]
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

    // Point ip_api to the mock server via env var. Use a scoped env helper
    // so tests avoid races and do not require unsafe blocks.
    let _guard = support::ScopedEnv::set_str("CHECK_VPN_TEST_URL", &server.url("/json"));

    // Call the function under test
    let isp = check_vpn::ip_api::get_isp().expect("get_isp failed");
    assert_eq!(isp, isp_value.to_string());

    mock.assert();
}
