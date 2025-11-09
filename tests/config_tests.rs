use check_vpn::config::Config;
use check_vpn::networking;

#[test]
fn validate_good_values() {
    let endpoints = vec!["8.8.8.8".to_string(), "google.com".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0], networking::DEFAULT_PORTS[1], networking::DEFAULT_PORTS[2]];
    let res = Config::validate_values(60, "Some ISP", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
    assert!(res.is_ok());
}

#[test]
fn validate_zero_interval() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(0, "ISP", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("interval must be greater than zero"));
}

#[test]
fn validate_empty_isp() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(60, "   ", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("isp_to_check must be a non-empty string"));
}

#[test]
fn validate_bad_action_type() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(60, "ISP", "unknown", "arg", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("vpn_lost_action_type must be one of"));
}

#[test]
fn validate_missing_arg_for_restart() {
    let endpoints = vec!["8.8.8.8".to_string()];
    let ports = vec![networking::DEFAULT_PORTS[0]];
    let res = Config::validate_values(60, "ISP", "restart-unit", "   ", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("vpn_lost_action_arg must be provided"));
}
