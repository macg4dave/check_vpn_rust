use std::fs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct IpApiResponse {
    status: Option<String>,
    isp: Option<String>,
}

#[test]
fn read_examples_ip_api_json_via_json_io() {
    // Reuse the same example file used elsewhere in the repo to ensure parity.
    let path = "examples/ip_api.json";
    // Simple smoke test: read the file and parse it with serde_json to match existing expectations
    let s = fs::read_to_string(path).expect("failed to read examples/ip_api.json");
    let parsed: IpApiResponse = serde_json::from_str(&s).expect("failed to parse json");
    assert_eq!(parsed.status.as_deref(), Some("success"));
    assert!(parsed.isp.is_some(), "expected isp present in examples/ip_api.json");
}

#[test]
fn json_io_read_and_roundtrip() {
    use check_vpn::json_io;
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Small { a: i32, b: String }

    let tmp = std::env::temp_dir().join("check_vpn_json_io_test.json");
    let path = tmp.to_str().expect("temp path utf8");

    let v = Small { a: 42, b: "hello".to_string() };
    json_io::write_json(&v, path).expect("write_json failed");
    let read: Small = json_io::read_json(path).expect("read_json failed");
    assert_eq!(v, read);

    let _ = std::fs::remove_file(path);
}

#[test]
fn json_io_streaming_reader_writer() {
    use check_vpn::json_io;
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Small { a: i32, b: String }

    let v = Small { a: 7, b: "stream".to_string() };
    let mut buf: Vec<u8> = Vec::new();
    json_io::write_json_to_writer(&v, &mut buf).expect("write_json_to_writer failed");
    let read: Small = json_io::read_json_from_reader(buf.as_slice()).expect("read_json_from_reader failed");
    assert_eq!(v, read);
}
