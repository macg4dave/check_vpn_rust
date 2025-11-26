use check_vpn::providers::{query_first_success, VpnInfoProvider};
use check_vpn::providers;
use httpmock::{MockServer, Method::GET};

// Simple wrapper implementing trait for test purposes around httpmock responses.

#[test]
fn ifconfig_co_parses_asn_org() {
    let server = MockServer::start();
    let body = include_str!("../examples/ifconfig.co.json");
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200).body(body);
    });
    // Use generic provider pointed at mock to validate parsing (asn_org fallback)
    let generic = providers::generic_json_provider::GenericJsonProvider::new(&server.url("/json")).unwrap();
    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(generic)];
    let id = query_first_success(&chain).expect("expected success");
    assert_eq!(id.isp, "Vodafone Limited");
}

#[test]
fn fallback_success_after_failure() {
    let server_good = MockServer::start();
    let server_bad = MockServer::start();
    let _good = server_good.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200).body("{\"isp\":\"GoodISP\"}");
    });
    let _bad = server_bad.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(500).body("");
    });

    let bad_p = providers::generic_json_provider::GenericJsonProvider::new(&server_bad.url("/json")).unwrap();
    let good_p = providers::generic_json_provider::GenericJsonProvider::new(&server_good.url("/json")).unwrap();

    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(bad_p), Box::new(good_p)];
    let id = query_first_success(&chain).expect("should fallback to good provider");
    assert_eq!(id.isp, "GoodISP");
}
