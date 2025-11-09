use std::time::{SystemTime, UNIX_EPOCH};
// PathBuf not needed yet

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
struct TestRound {
    name: String,
    value: u64,
}

#[test]
fn write_and_read_roundtrip_simple() {
    let obj = TestRound { name: "alice".to_string(), value: 42 };

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("check_vpn_test_round_{}.xml", now));
    let path_str = path.to_str().expect("failed to get path string");

    // write and read back using xml_io helpers
    check_vpn::xml_io::write_xml(&obj, path_str).expect("write_xml failed");
    let got: TestRound = check_vpn::xml_io::read_xml(path_str).expect("read_xml failed");
    let _ = std::fs::remove_file(path_str);

    assert_eq!(obj, got, "roundtrip should preserve the struct");
}

#[test]
fn read_config_parses_basic_fields() {
    // Create a minimal XML string for Config that omits vectors to avoid serialization quirks.
    let xml = r#"
    <config>
      <interval>123</interval>
      <isp_to_check>Test ISP</isp_to_check>
      <vpn_lost_action_type>command</vpn_lost_action_type>
      <vpn_lost_action_arg>/bin/true</vpn_lost_action_arg>
      <dry_run>true</dry_run>
    </config>
    "#;

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!("check_vpn_test_config_read_{}.xml", now));
    let path_str = path.to_str().expect("failed to get path string");

    std::fs::write(path_str, xml).expect("failed to write xml file");
    let cfg: check_vpn::config::Config = check_vpn::xml_io::read_xml(path_str).expect("read_xml failed");
    let _ = std::fs::remove_file(path_str);

    assert_eq!(cfg.interval, Some(123));
    assert_eq!(cfg.isp_to_check.unwrap(), "Test ISP");
    assert_eq!(cfg.vpn_lost_action_type.unwrap(), "command");
    assert_eq!(cfg.vpn_lost_action_arg.unwrap(), "/bin/true");
    assert_eq!(cfg.dry_run, Some(true));
}
