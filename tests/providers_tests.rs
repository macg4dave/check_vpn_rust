use check_vpn::providers;
use check_vpn::providers::{query_first_success, VpnInfoProvider};
use httpmock::{Method::GET, MockServer};

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
    let generic =
        providers::generic_json_provider::GenericJsonProvider::new(&server.url("/json")).unwrap();
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

    let bad_p =
        providers::generic_json_provider::GenericJsonProvider::new(&server_bad.url("/json"))
            .unwrap();
    let good_p =
        providers::generic_json_provider::GenericJsonProvider::new(&server_good.url("/json"))
            .unwrap();

    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(bad_p), Box::new(good_p)];
    let id = query_first_success(&chain).expect("should fallback to good provider");
    assert_eq!(id.isp, "GoodISP");
}

#[test]
fn ip_api_provider_uses_https_default() {
    let p = providers::ip_api_provider::IpApiProvider::new_default().unwrap();
    assert!(
        p.endpoint().starts_with("https://"),
        "ip-api provider should default to HTTPS endpoint"
    );
}

#[test]
fn generic_provider_rejects_large_response() {
    struct ScopedEnv {
        prev: Option<std::ffi::OsString>,
    }
    impl ScopedEnv {
        fn set_max_bytes(v: &str) -> Self {
            let prev = std::env::var_os("CHECK_VPN_MAX_RESPONSE_BYTES");
            std::env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", v);
            ScopedEnv { prev }
        }
    }
    impl Drop for ScopedEnv {
        fn drop(&mut self) {
            if let Some(p) = self.prev.take() {
                std::env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", p);
            } else {
                std::env::remove_var("CHECK_VPN_MAX_RESPONSE_BYTES");
            }
        }
    }

    let _guard = ScopedEnv::set_max_bytes("1024");
    let server = MockServer::start();
    let body = "x".repeat(2048);
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .header("content-length", "2048")
            .body(body.as_str());
    });

    let p = providers::generic_json_provider::GenericJsonProvider::new(&server.url("/json"))
        .expect("provider init");
    let chain: Vec<Box<dyn VpnInfoProvider>> = vec![Box::new(p)];
    let res = query_first_success(&chain);
    assert!(res.is_err(), "expected large response to be rejected");
}

#[test]
fn ifconfig_provider_rejects_large_response() {
    struct ScopedEnv {
        prev: Option<std::ffi::OsString>,
    }
    impl ScopedEnv {
        fn set_max_bytes(v: &str) -> Self {
            let prev = std::env::var_os("CHECK_VPN_MAX_RESPONSE_BYTES");
            std::env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", v);
            ScopedEnv { prev }
        }
    }
    impl Drop for ScopedEnv {
        fn drop(&mut self) {
            if let Some(p) = self.prev.take() {
                std::env::set_var("CHECK_VPN_MAX_RESPONSE_BYTES", p);
            } else {
                std::env::remove_var("CHECK_VPN_MAX_RESPONSE_BYTES");
            }
        }
    }

    let _guard = ScopedEnv::set_max_bytes("1024");
    let server = MockServer::start();
    let body = "x".repeat(4096);
    let _m = server.mock(|when, then| {
        when.method(GET).path("/json");
        then.status(200)
            .header("content-type", "application/json")
            .header("content-length", "4096")
            .body(body.as_str());
    });

    let p = providers::ifconfig_co_provider::IfconfigCoProvider::new_with_url(&server.url("/json"))
        .unwrap();
    let res = p.query();
    assert!(
        res.is_err(),
        "expected large ifconfig response to be rejected"
    );
}
