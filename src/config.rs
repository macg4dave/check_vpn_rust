use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Clone)]
#[serde(rename = "config")]
pub struct Config {
    pub interval: Option<u64>,
    pub isp_to_check: Option<String>,
    pub vpn_lost_action_type: Option<String>,
    pub vpn_lost_action_arg: Option<String>,
    pub dry_run: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            interval: Some(60),
            isp_to_check: Some("Hutchison 3G UK Ltd".to_string()),
            vpn_lost_action_type: Some("reboot".to_string()),
            vpn_lost_action_arg: Some("/sbin/shutdown -r now".to_string()),
            dry_run: Some(false),
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
            return Self::load_from_path(&path);
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
        let contents = fs::read_to_string(path).with_context(|| format!("failed to read config file: {}", path))?;
        let c: Config = serde_xml_rs::from_str(&contents).context("failed to parse XML config")?;
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
