//! Configuration types and helpers for check_vpn.
//!
//! This module was reorganized to separate serde helpers and validation
//! concerns into small focused modules. Public API is preserved so existing
//! callers (e.g. `crate::config::Config`, `EffectiveConfig` and
//! `ValidationErrors`) continue to work.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub mod serde_helpers;
pub mod validation;

pub use validation::ValidationErrors;
pub use serde_helpers::serialize_option_bool;

/// Exit codes used by the application for specific error classes.
pub const EXIT_INVALID_CONFIG: i32 = 2;
/// DNS or name-resolution failure while attempting connectivity checks
pub const EXIT_CONNECTIVITY_DNS: i32 = 3;
/// Generic connectivity failure (unreachable/timeouts) when considered fatal
pub const EXIT_CONNECTIVITY_FAILURE: i32 = 4;
/// Failed to determine ISP (IP API) when considered fatal
pub const EXIT_ISP_FAILURE: i32 = 5;

#[cfg_attr(feature = "xml_strict", derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq))]
#[cfg_attr(not(feature = "xml_strict",), derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq))]
#[serde(rename = "config")]
#[cfg_attr(feature = "xml_strict", serde(deny_unknown_fields))]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isp_to_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpn_lost_action_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vpn_lost_action_arg: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::config::serialize_option_bool"
    )]
    pub dry_run: Option<bool>,
    /// If true, exit with non-zero codes on networking/ISP errors even in long-running mode
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::config::serialize_option_bool"
    )]
    pub exit_on_error: Option<bool>,
    // Connectivity-related configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_endpoints: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_ports: Option<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_timeout_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity_retries: Option<usize>,
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
    pub connectivity_retries: usize,
    pub run_once: bool,
    pub exit_on_error: bool,
}

impl Config {
    /// Merge CLI args and Config into an EffectiveConfig (CLI overrides Config)
    pub fn merge_with_args(&self, args: &crate::cli::Args) -> EffectiveConfig {
        use crate::networking;

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
            .unwrap_or_else(|| vec![networking::DEFAULT_PORTS[0], networking::DEFAULT_PORTS[1], networking::DEFAULT_PORTS[2]]);

        let connectivity_timeout_secs = args
            .connectivity_timeout_secs
            .or(self.connectivity_timeout_secs)
            .unwrap_or(networking::DEFAULT_TIMEOUT_SECS);

        let connectivity_retries = args
            .connectivity_retries
            .or(self.connectivity_retries)
            .unwrap_or(networking::DEFAULT_RETRIES);

        let exit_on_error = if args.exit_on_error { true } else { self.exit_on_error.unwrap_or(false) };

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
            connectivity_retries,
            run_once,
            exit_on_error,
        }
    }
}

impl Config {
    /// Backwards-compatible wrapper that delegates to the validation module.
    ///
    /// Kept as an associated function for compatibility with call sites that
    /// expect `Config::validate_values(...)`.
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
        validation::validate_values(
            interval,
            isp,
            action_type,
            action_arg,
            connectivity_endpoints,
            connectivity_ports,
            connectivity_timeout_secs,
            connectivity_retries,
        )
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
            connectivity_retries: Some(crate::networking::DEFAULT_RETRIES),
            exit_on_error: Some(false),
        }
    }
}

impl Config {
    /// Load configuration from XML file. Order of lookup:
    /// 1. `CHECK_VPN_CONFIG` env var path
    /// 2. `./check_vpn.xml`
    /// 3. `/etc/check_vpn/config.xml`
    ///    If no file is found, returns the default config.
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
}
