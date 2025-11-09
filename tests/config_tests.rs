use check_vpn::config::Config;

#[test]
fn validate_good_values() {
    let res = Config::validate_values(60, "Some ISP", "reboot", "");
    assert!(res.is_ok());
}

#[test]
fn validate_zero_interval() {
    let res = Config::validate_values(0, "ISP", "reboot", "");
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("interval must be greater than zero"));
}

#[test]
fn validate_empty_isp() {
    let res = Config::validate_values(60, "   ", "reboot", "");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("isp_to_check must be a non-empty string"));
}

#[test]
fn validate_bad_action_type() {
    let res = Config::validate_values(60, "ISP", "unknown", "arg");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("vpn_lost_action_type must be one of"));
}

#[test]
fn validate_missing_arg_for_restart() {
    let res = Config::validate_values(60, "ISP", "restart-unit", "   ");
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("vpn_lost_action_arg must be provided"));
}
