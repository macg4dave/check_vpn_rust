use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
// std::fs is no longer used here; xml_io handles file operations.
// (Left intentionally blank to avoid unused-import warnings.)

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "config")]
pub struct Config {
    pub interval: Option<u64>,
    pub isp_to_check: Option<String>,
    pub vpn_lost_action_type: Option<String>,
    pub vpn_lost_action_arg: Option<String>,
    pub dry_run: Option<bool>,
    // Connectivity-related configuration
    pub connectivity_endpoints: Option<Vec<String>>,
    pub connectivity_ports: Option<Vec<u16>>,
    pub connectivity_timeout_secs: Option<u64>,
}

/// Effective configuration after merging CLI args and XML config/defaults.
#[derive(Debug, Clone)]
pub struct EffectiveConfig {
    pub interval: u64,
    pub isp_to_check: String,
    pub action_type: String,
    pub action_arg: String,
    pub dry_run: bool,
    pub connectivity_endpoints: Vec<String>,
    pub connectivity_ports: Vec<u16>,
    pub connectivity_timeout_secs: u64,
    pub run_once: bool,
}

impl Config {
    /// Merge CLI args and Config into an EffectiveConfig (CLI overrides Config)
    pub fn merge_with_args(&self, args: &crate::cli::Args) -> EffectiveConfig {
        let interval = args.interval.or(self.interval).unwrap_or(60);
        let isp_to_check = args
            .isp_to_check
            .clone()
            .or_else(|| self.isp_to_check.clone())
            .unwrap_or_else(|| "Hutchison 3G UK Ltd".to_string());

        let action_type = args
            .vpn_lost_action_type
            .clone()
            .or_else(|| self.vpn_lost_action_type.clone())
            .unwrap_or_else(|| "reboot".to_string());

        let action_arg = args
            .vpn_lost_action_arg
            .clone()
            .or_else(|| self.vpn_lost_action_arg.clone())
            .unwrap_or_else(|| "/sbin/shutdown -r now".to_string());

        let dry_run = if args.dry_run { true } else { self.dry_run.unwrap_or(false) };

        let connectivity_endpoints = args
            .connectivity_endpoints
            .clone()
            .or_else(|| self.connectivity_endpoints.clone())
            .unwrap_or_else(|| vec!["8.8.8.8".to_string(), "google.com".to_string()]);

        let connectivity_ports = args
            .connectivity_ports
            .clone()
            .or_else(|| self.connectivity_ports.clone())
            .unwrap_or_else(|| vec![crate::networking::DEFAULT_PORTS[0], crate::networking::DEFAULT_PORTS[1], crate::networking::DEFAULT_PORTS[2]]);

        let connectivity_timeout_secs = args
            .connectivity_timeout_secs
            .or(self.connectivity_timeout_secs)
            .unwrap_or(crate::networking::DEFAULT_TIMEOUT_SECS);

        let run_once = args.run_once;

        EffectiveConfig {
            interval,
            isp_to_check,
            action_type,
            action_arg,
            dry_run,
            connectivity_endpoints,
            connectivity_ports,
            connectivity_timeout_secs,
            run_once,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval: Some(60),
            isp_to_check: Some("Hutchison 3G UK Ltd".to_string()),
            vpn_lost_action_type: Some("reboot".to_string()),
            vpn_lost_action_arg: Some("/sbin/shutdown -r now".to_string()),
            dry_run: Some(false),
            connectivity_endpoints: Some(vec!["8.8.8.8".to_string(), "google.com".to_string()]),
            connectivity_ports: Some(vec![crate::networking::DEFAULT_PORTS[0], crate::networking::DEFAULT_PORTS[1], crate::networking::DEFAULT_PORTS[2]]),
            connectivity_timeout_secs: Some(crate::networking::DEFAULT_TIMEOUT_SECS),
        }
    }
}

impl Config {
    /// Load configuration from XML file. Order of lookup:
    /// 1. `CHECK_VPN_CONFIG` env var path
    /// 2. `./check_vpn.xml`
    /// 3. `/etc/check_vpn/config.xml`
    /// If no file is found, returns the default config.
    pub fn load() -> Result<Self> {
        if let Ok(path) = std::env::var("CHECK_VPN_CONFIG") {
            return crate::xml_io::read_xml(&path).context("failed to read config via xml_io");
        }

        let cwd = std::env::current_dir().context("failed to get current dir")?;
        let local = cwd.join("check_vpn.xml");
        if local.exists() {
            return Self::load_from_path(local.to_str().unwrap());
        }

        let etc = std::path::Path::new("/etc/check_vpn/config.xml");
        if etc.exists() {
            return Self::load_from_path(etc.to_str().unwrap());
        }

        Ok(Config::default())
    }

    fn load_from_path(path: &str) -> Result<Self> {
        // keep this helper for backwards compat if other callers used it; delegate to xml_io
        let c: Config = crate::xml_io::read_xml(path).context("failed to read config via xml_io")?;
        Ok(c)
    }

    /// Validate effective configuration values. Returns Ok(()) if valid or Err(vec_of_errors).
    pub fn validate_values(interval: u64, isp: &str, action_type: &str, action_arg: &str) -> Result<()> {
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

        if errors.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!(errors.join("; ")))
        }
    }
}
