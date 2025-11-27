use check_vpn::config::Config;
use check_vpn::networking;
use clap::Parser;
// env mutation is handled via tests/support::ScopedEnv
mod support;
use serial_test::serial;

#[test]
fn validate_good_values() {
    let endpoints = vec!["8.8.8.8".to_string(), "google.com".to_string()];
    let ports = vec![
        networking::DEFAULT_PORTS[0],
        networking::DEFAULT_PORTS[1],
        networking::DEFAULT_PORTS[2],
    ];
    let res = Config::validate_values(
        60,
        "Some ISP",
        "reboot",
        "",
        &endpoints,
        &ports,
        networking::DEFAULT_TIMEOUT_SECS,
        networking::DEFAULT_RETRIES,
    );
    assert!(res.is_ok());
}

#[test]
fn validate_zero_interval() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(
        0,
        "ISP",
        "reboot",
        "",
        &endpoints,
        &ports,
        networking::DEFAULT_TIMEOUT_SECS,
        networking::DEFAULT_RETRIES,
    );
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("interval must be greater than zero"));
}

#[test]
fn validate_empty_isp() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(
        60,
        "   ",
        "reboot",
        "",
        &endpoints,
        &ports,
        networking::DEFAULT_TIMEOUT_SECS,
        networking::DEFAULT_RETRIES,
    );
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("isp_to_check must be a non-empty string"));
}

#[test]
fn validate_bad_action_type() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(
        60,
        "ISP",
        "unknown",
        "arg",
        &endpoints,
        &ports,
        networking::DEFAULT_TIMEOUT_SECS,
        networking::DEFAULT_RETRIES,
    );
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("vpn_lost_action_type must be one of"));
}

#[test]
fn validate_missing_arg_for_restart() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(
        60,
        "ISP",
        "restart-unit",
        "   ",
        &endpoints,
        &ports,
        networking::DEFAULT_TIMEOUT_SECS,
        networking::DEFAULT_RETRIES,
    );
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("vpn_lost_action_arg must be provided"));
}

#[test]
fn missing_required_field_uses_default_on_merge() {
    // Create XML missing the isp_to_check field; Config fields are optional and
    // merge_with_args should apply defaults for missing values.
    let xml = r#"<config><interval>30</interval></config>"#;
    let tmpf = support::create_temp_file_with_contents("missing_field", xml.as_bytes());
    let path_str = tmpf.path().to_str().unwrap();

    let cfg = check_vpn::xml_io::read_xml::<check_vpn::config::Config>(path_str)
        .expect("read xml should succeed");
    // cleanup handled by NamedTempFile drop

    // Build default Args via clap (no overrides). Avoid manual struct literal to remain stable when fields evolve.
    let args = check_vpn::cli::Args::parse_from(["check_vpn"].as_slice());

    let eff = cfg.merge_with_args(&args);
    // Default ISP from Config::merge_with_args is "Hutchison 3G UK Ltd"
    assert_eq!(eff.isp_to_check, "Hutchison 3G UK Ltd");
}

#[test]
#[serial(env)]
fn env_var_config_override() {
    // Create a small config file and point CHECK_VPN_CONFIG at it
    let xml = r#"<config><interval>77</interval><isp_to_check>Env ISP</isp_to_check></config>"#;
    let (_td, p) = support::temp_dir_with_file("cfg_override", "check_vpn_env_override.xml", xml.as_bytes());
    let path_str = p.to_str().unwrap();

    // Use a scoped environment helper to ensure the env var is restored.
    let _guard = support::ScopedEnv::set_str("CHECK_VPN_CONFIG", path_str);
    let cfg = Config::load().expect("Config::load should succeed when pointed to temp file");
    // `TempDir` `_td` is dropped and cleans up the temporary file

    assert_eq!(cfg.interval, Some(77));
    assert_eq!(cfg.isp_to_check.unwrap(), "Env ISP");
}
