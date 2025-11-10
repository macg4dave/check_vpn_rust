use httpmock::{MockServer, Method::GET};
use check_vpn::providers::{query_first_success, VpnInfoProvider};

#[test]
fn custom_json_key_extraction() {
    let server = MockServer::start();
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200).body("{ \"asn\": \"ASN123\" }");
    });

    // Provider with preferred key 'asn'
    let gp = check_vpn::providers::generic_json_provider::GenericJsonProvider::new(&server.url("/json"))
        .unwrap()
        .with_key(Some("asn".to_string()));
    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(gp)];
    let id = query_first_success(&chain).expect("expected success");
    assert_eq!(id.isp, "ASN123");
}

#[test]
fn custom_json_key_fallback_when_missing() {
    let server = MockServer::start();
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200).body("{ \"org\": \"OrgValue\" }");
    });

    // Preferred key 'asn' missing, should fallback to org
    let gp = check_vpn::providers::generic_json_provider::GenericJsonProvider::new(&server.url("/json"))
        .unwrap()
        .with_key(Some("asn".to_string()));
    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(gp)];
    let id = query_first_success(&chain).expect("expected fallback success");
    assert_eq!(id.isp, "OrgValue");
}
