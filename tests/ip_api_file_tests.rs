use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    status: Option<String>,
    isp: Option<String>,
}

#[test]
fn read_examples_ip_api_json() {
    let path = "examples/ip_api.json";
    let s = fs::read_to_string(path).expect("failed to read examples/ip_api.json");
    let parsed: IpApiResponse = serde_json::from_str(&s).expect("failed to parse json");
    // Expect status success and that `isp` exists in the saved example.
    assert_eq!(parsed.status.as_deref(), Some("success"));
    assert!(
        parsed.isp.is_some(),
        "expected isp present in examples/ip_api.json"
    );
}
