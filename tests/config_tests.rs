use check_vpn::config::Config;
use check_vpn::networking;
use std::env;
use std::fs;

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
    let mut path = std::env::temp_dir();
    path.push(format!(
        "check_vpn_missing_field_{}.xml",
        std::time::Instant::now().elapsed().as_nanos()
    ));
    let path_str = path.to_str().unwrap();
    fs::write(path_str, xml).expect("write temp xml");

    let cfg = check_vpn::xml_io::read_xml::<check_vpn::config::Config>(path_str)
        .expect("read xml should succeed");
    let _ = fs::remove_file(path_str);

    // Build a default Args (no overrides)
    let args = check_vpn::cli::Args {
        interval: None,
        isp_to_check: None,
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: false,
        connectivity_endpoints: None,
        connectivity_ports: None,
        connectivity_timeout_secs: None,
        connectivity_retries: None,
        run_once: false,
        exit_on_error: false,
        verbose: 0,
        config: None,
        enable_metrics: false,
        metrics_addr: "0.0.0.0:9090".to_string(),
    };

    let eff = cfg.merge_with_args(&args);
    // Default ISP from Config::merge_with_args is "Hutchison 3G UK Ltd"
    assert_eq!(eff.isp_to_check, "Hutchison 3G UK Ltd");
}

#[test]
fn env_var_config_override() {
    // Create a small config file and point CHECK_VPN_CONFIG at it
    let xml = r#"<config><interval>77</interval><isp_to_check>Env ISP</isp_to_check></config>"#;
    let mut path = std::env::temp_dir();
    path.push(format!(
        "check_vpn_env_override_{}.xml",
        std::time::Instant::now().elapsed().as_nanos()
    ));
    let path_str = path.to_str().unwrap();
    fs::write(path_str, xml).expect("write temp xml");

    // Some targets mark `set_var` as unsafe; use an unsafe block for portability.
    unsafe {
        env::set_var("CHECK_VPN_CONFIG", path_str);
    }
    let cfg = Config::load().expect("Config::load should succeed when pointed to temp file");
    // cleanup
    unsafe {
        env::remove_var("CHECK_VPN_CONFIG");
    }
    let _ = fs::remove_file(path_str);

    assert_eq!(cfg.interval, Some(77));
    assert_eq!(cfg.isp_to_check.unwrap(), "Env ISP");
}
