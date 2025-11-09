use std::fmt;

/// ValidationErrors represents one or more config validation problems.
#[derive(Debug, PartialEq, Eq)]
pub struct ValidationErrors(pub Vec<String>);

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join("; "))
    }
}

impl std::error::Error for ValidationErrors {}

/// Validate effective configuration values. Returns Ok(()) if valid or Err(errors).
#[allow(clippy::too_many_arguments)]
pub fn validate_values(
    interval: u64,
    isp: &str,
    action_type: &str,
    action_arg: &str,
    connectivity_endpoints: &[String],
    connectivity_ports: &[u16],
    connectivity_timeout_secs: u64,
    connectivity_retries: usize,
) -> std::result::Result<(), ValidationErrors> {
    let mut errors: Vec<String> = Vec::new();

    if interval == 0 {
        errors.push("interval must be greater than zero".to_string());
    }

    if isp.trim().is_empty() {
        errors.push("isp_to_check must be a non-empty string".to_string());
    }

    let allowed = ["reboot", "restart-unit", "command"];
    if !allowed.contains(&action_type) {
        errors.push(format!("vpn_lost_action_type must be one of: {}", allowed.join(", ")));
    }

    // For restart-unit and command, ensure arg is non-empty
    if (action_type == "restart-unit" || action_type == "command") && action_arg.trim().is_empty() {
        errors.push("vpn_lost_action_arg must be provided for restart-unit and command action types".to_string());
    }

    // Connectivity checks: endpoints and ports must be present and timeout/retries sensible
    if connectivity_endpoints.is_empty() {
        errors.push("connectivity_endpoints must include at least one endpoint".to_string());
    } else if connectivity_endpoints.iter().any(|s| s.trim().is_empty()) {
        errors.push("connectivity_endpoints contains an empty string".to_string());
    }

    if connectivity_ports.is_empty() {
        errors.push("connectivity_ports must include at least one port".to_string());
    }

    if connectivity_timeout_secs == 0 {
        errors.push("connectivity_timeout_secs must be greater than zero".to_string());
    }

    if connectivity_retries == 0 {
        errors.push("connectivity_retries must be at least 1".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_values;
    use crate::networking;

    #[test]
    fn validate_values_happy_path() {
        let endpoints = vec!["8.8.8.8".to_string(), "google.com".to_string()];
        let ports = vec![networking::DEFAULT_PORTS[0], networking::DEFAULT_PORTS[1]];
        let res = validate_values(60, "Some ISP", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
        assert!(res.is_ok());
    }

    #[test]
    fn validate_zero_interval() {
        let endpoints = vec!["8.8.8.8".to_string()];
        let ports = vec![networking::DEFAULT_PORTS[0]];
        let res = validate_values(0, "ISP", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        assert!(err.contains("interval must be greater than zero"));
    }

    #[test]
    fn validate_empty_isp() {
        let endpoints = vec!["8.8.8.8".to_string()];
        let ports = vec![networking::DEFAULT_PORTS[0]];
        let res = validate_values(60, "   ", "reboot", "", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("isp_to_check must be a non-empty string"));
    }

    #[test]
    fn validate_bad_action_type() {
        let endpoints = vec!["8.8.8.8".to_string()];
        let ports = vec![networking::DEFAULT_PORTS[0]];
        let res = validate_values(60, "ISP", "unknown", "arg", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("vpn_lost_action_type must be one of"));
    }

    #[test]
    fn validate_missing_arg_for_restart() {
        let endpoints = vec!["8.8.8.8".to_string()];
        let ports = vec![networking::DEFAULT_PORTS[0]];
        let res = validate_values(60, "ISP", "restart-unit", "   ", &endpoints, &ports, networking::DEFAULT_TIMEOUT_SECS, networking::DEFAULT_RETRIES);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("vpn_lost_action_arg must be provided"));
    }
}
