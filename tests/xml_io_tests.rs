mod support;
// PathBuf not needed yet

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
struct TestRound {
    name: String,
    value: u64,
}

#[test]
fn write_and_read_roundtrip_simple() {
    let obj = TestRound {
        name: "alice".to_string(),
        value: 42,
    };

    let (_td, p) = support::temp_dir_with_child("xml_roundtrip", "check_vpn_test_round.xml");
    let path_str = p.to_str().expect("failed to get path string");

    // write and read back using xml_io helpers
    check_vpn::xml_io::write_xml(&obj, path_str).expect("write_xml failed");
    let got: TestRound = check_vpn::xml_io::read_xml(path_str).expect("read_xml failed");
    // `_td` drops and cleans up

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

    let (_td, p) = support::temp_dir_with_child("xml_read", "check_vpn_test_config_read.xml");
    let path_str = p.to_str().expect("failed to get path string");
    std::fs::write(path_str, xml).expect("failed to write xml file");
    let cfg: check_vpn::config::Config =
        check_vpn::xml_io::read_xml(path_str).expect("read_xml failed");
    // `_td` drops and cleans up

    assert_eq!(cfg.interval, Some(123));
    assert_eq!(cfg.isp_to_check.unwrap(), "Test ISP");
    assert_eq!(cfg.vpn_lost_action_type.unwrap(), "command");
    assert_eq!(cfg.vpn_lost_action_arg.unwrap(), "/bin/true");
    assert_eq!(cfg.dry_run, Some(true));
}

#[test]
fn roundtrip_example_config() {
    // Use the provided example config file, deserialize, serialize to temp, read back
    let example_path = std::path::Path::new("examples/check_vpn.xml");
    assert!(
        example_path.exists(),
        "examples/check_vpn.xml must exist for this test"
    );

    let cfg1: check_vpn::config::Config =
        check_vpn::xml_io::read_xml(example_path.to_str().unwrap())
            .expect("failed to read example config");

    let (_td, p) = support::temp_dir_with_child("xml_example", "check_vpn_example_roundtrip.xml");
    let path_str = p.to_str().expect("failed to get path string");

    check_vpn::xml_io::write_xml(&cfg1, path_str).expect("write_xml failed");
    let cfg2: check_vpn::config::Config =
        check_vpn::xml_io::read_xml(path_str).expect("read_xml failed");
    // `_td` drops and cleans up

    assert_eq!(
        cfg1, cfg2,
        "roundtrip of example config should preserve values"
    );
}

#[test]
fn pretty_xml_inserts_newlines_and_indentation() {
    let cfg = check_vpn::config::Config::default();
    let (_td, p) = support::temp_dir_with_child("xml_pretty", "check_vpn_pretty.xml");
    let path_str = p.to_str().expect("failed to get path string");

    check_vpn::xml_io::write_xml(&cfg, path_str).expect("write_xml failed");
    let contents = std::fs::read_to_string(path_str).expect("read back");

    assert!(
        contents.contains('\n'),
        "pretty XML should contain newlines"
    );
    assert!(
        contents.contains("  <"),
        "pretty XML should contain indentation"
    );
}

#[test]
fn malformed_xml_errors_gracefully() {
    // Missing closing tag for <config> and interval not closed properly
    let bad_xml = r#"<config><interval>60<interval><isp_to_check>BadISP</isp_to_check>"#;
    let (_td, p) = support::temp_dir_with_child("xml_bad", "check_vpn_bad_xml.xml");
    let path_str = p.to_str().expect("failed to get path string");
    std::fs::write(path_str, bad_xml).expect("failed to write bad xml");

    let err = check_vpn::xml_io::read_xml::<check_vpn::config::Config>(path_str)
        .expect_err("expected parse error");
    // `_td` drops and cleans up

    let msg = format!("{}", err);
    assert!(
        msg.contains("failed to parse xml"),
        "error should include context prefix: {}",
        msg
    );
    // Position hints are best-effort; ensure message doesn't panic
    // (Line/column may or may not appear depending on backend; just ensure non-empty)
    assert!(!msg.trim().is_empty(), "error message should not be empty");
}

// Strict unknown field handling when xml_strict feature is enabled
#[cfg(feature = "xml_strict")]
#[test]
fn strict_unknown_field_errors() {
    let bad_xml = r#"<config><interval>60</interval><unknownField>abc</unknownField></config>"#;
    let (_td, p) = support::temp_dir_with_child("xml_bad_unknown", "check_vpn_bad_unknown.xml");
    let path_str = p.to_str().expect("failed to get path string");
    std::fs::write(path_str, bad_xml).expect("failed to write bad xml");
    let err = check_vpn::xml_io::read_xml::<check_vpn::config::Config>(path_str)
        .expect_err("expected unknown field error");
    // `_td` drops and cleans up
    let msg = format!("{}", err);
    assert!(
        msg.contains("unknown"),
        "strict mode should error on unknown field: {}",
        msg
    );
}
