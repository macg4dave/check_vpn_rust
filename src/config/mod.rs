//! Partial file configuration, resolved runtime settings and pinned configuration sources.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub mod serde_helpers;
pub mod validation;

pub use serde_helpers::serialize_option_bool;
pub use validation::ValidationErrors;

/// Compatibility alias; exit code definitions live in error_types.
pub use crate::error_types::EXIT_INVALID_CONFIG;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename = "config")]
#[cfg_attr(feature = "xml_strict", serde(deny_unknown_fields))]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isp_aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<ProviderSpec>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_cooldown_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_timeout_secs: Option<u64>,
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
    /// Exit on lookup and recovery errors even in long-running mode
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

    // Provider selection and customization
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::config::serialize_option_bool"
    )]
    pub enable_ip_api: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::config::serialize_option_bool"
    )]
    pub enable_ifconfig_co: Option<bool>,
    /// Additional provider URLs to try (queried before built-ins, in order)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_urls: Option<Vec<String>>,

    /// Convenience: a single custom JSON server to query first (before provider_urls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_json_server: Option<String>,
    /// The JSON key to extract from the custom server (e.g., "asn", "org", "isp")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_json_key: Option<String>,
}

/// Effective configuration after merging CLI args and XML config/defaults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveConfig {
    pub isp_aliases: Vec<String>,
    pub providers: Vec<ProviderSpec>,
    pub recovery_cooldown_secs: u64,
    pub action_timeout_secs: u64,
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
    // Providers
    pub enable_ip_api: bool,
    pub enable_ifconfig_co: bool,
    pub provider_urls: Vec<String>,
    pub custom_json_server: Option<String>,
    pub custom_json_key: Option<String>,
}

/// Ordered JSON source. Existing custom_json_server/provider_urls remain aliases.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "xml_strict", serde(deny_unknown_fields))]
pub struct ProviderSpec {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

impl Default for EffectiveConfig {
    fn default() -> Self {
        Self {
            interval: 60,
            isp_to_check: "Hutchison 3G UK Ltd".into(),
            action_type: "log".into(),
            action_arg: String::new(),
            dry_run: false,
            connectivity_endpoints: vec!["8.8.8.8".into(), "google.com".into()],
            connectivity_ports: crate::networking::DEFAULT_PORTS.to_vec(),
            connectivity_timeout_secs: crate::networking::DEFAULT_TIMEOUT_SECS,
            connectivity_retries: crate::networking::DEFAULT_RETRIES,
            run_once: false,
            exit_on_error: false,
            enable_ip_api: true,
            enable_ifconfig_co: true,
            provider_urls: vec![],
            custom_json_server: None,
            custom_json_key: None,
            isp_aliases: vec![],
            providers: vec![],
            recovery_cooldown_secs: 60,
            action_timeout_secs: 30,
        }
    }
}

impl Config {
    /// Merge CLI > file > the single set of runtime defaults.
    pub fn merge_with_args(&self, args: &crate::cli::Args) -> EffectiveConfig {
        let d = EffectiveConfig::default();
        EffectiveConfig {
            interval: args.interval.or(self.interval).unwrap_or(d.interval),
            isp_to_check: args
                .isp_to_check
                .clone()
                .or_else(|| self.isp_to_check.clone())
                .unwrap_or(d.isp_to_check),
            action_type: args
                .vpn_lost_action_type
                .clone()
                .or_else(|| self.vpn_lost_action_type.clone())
                .unwrap_or(d.action_type),
            action_arg: args
                .vpn_lost_action_arg
                .clone()
                .or_else(|| self.vpn_lost_action_arg.clone())
                .unwrap_or(d.action_arg),
            dry_run: args.dry_run || self.dry_run.unwrap_or(d.dry_run),
            exit_on_error: args.exit_on_error || self.exit_on_error.unwrap_or(d.exit_on_error),
            run_once: args.run_once,
            connectivity_endpoints: args
                .connectivity_endpoints
                .clone()
                .or_else(|| self.connectivity_endpoints.clone())
                .unwrap_or(d.connectivity_endpoints),
            connectivity_ports: args
                .connectivity_ports
                .clone()
                .or_else(|| self.connectivity_ports.clone())
                .unwrap_or(d.connectivity_ports),
            connectivity_timeout_secs: args
                .connectivity_timeout_secs
                .or(self.connectivity_timeout_secs)
                .unwrap_or(d.connectivity_timeout_secs),
            connectivity_retries: args
                .connectivity_retries
                .or(self.connectivity_retries)
                .unwrap_or(d.connectivity_retries),
            enable_ip_api: !args.disable_ip_api && self.enable_ip_api.unwrap_or(d.enable_ip_api),
            enable_ifconfig_co: !args.disable_ifconfig_co
                && self.enable_ifconfig_co.unwrap_or(d.enable_ifconfig_co),
            provider_urls: args
                .provider_urls
                .clone()
                .or_else(|| self.provider_urls.clone())
                .unwrap_or_default(),
            custom_json_server: args
                .custom_json_server
                .clone()
                .or_else(|| self.custom_json_server.clone()),
            custom_json_key: args
                .custom_json_key
                .clone()
                .or_else(|| self.custom_json_key.clone()),
            isp_aliases: args
                .isp_aliases
                .clone()
                .or_else(|| self.isp_aliases.clone())
                .unwrap_or_default(),
            providers: self.providers.clone().unwrap_or_default(),
            recovery_cooldown_secs: args
                .recovery_cooldown_secs
                .or(self.recovery_cooldown_secs)
                .unwrap_or(d.recovery_cooldown_secs),
            action_timeout_secs: args
                .action_timeout_secs
                .or(self.action_timeout_secs)
                .unwrap_or(d.action_timeout_secs),
        }
    }
}

impl Config {
    /// Backwards-compatible wrapper that delegates to the validation module.
    ///
    /// Kept as an associated function for compatibility with call sites that
    /// expect `Config::validate_values(...)`.
    pub fn validate_values(eff: &EffectiveConfig) -> std::result::Result<(), ValidationErrors> {
        validation::validate_values(eff)
    }
}

impl Default for Config {
    fn default() -> Self {
        let d = EffectiveConfig::default();
        Self {
            interval: Some(d.interval),
            isp_to_check: Some(d.isp_to_check),
            vpn_lost_action_type: Some(d.action_type),
            vpn_lost_action_arg: Some(d.action_arg),
            dry_run: Some(d.dry_run),
            exit_on_error: Some(d.exit_on_error),
            connectivity_endpoints: Some(d.connectivity_endpoints),
            connectivity_ports: Some(d.connectivity_ports),
            connectivity_timeout_secs: Some(d.connectivity_timeout_secs),
            connectivity_retries: Some(d.connectivity_retries),
            enable_ip_api: Some(d.enable_ip_api),
            enable_ifconfig_co: Some(d.enable_ifconfig_co),
            provider_urls: None,
            custom_json_server: None,
            custom_json_key: None,
            isp_aliases: None,
            providers: None,
            recovery_cooldown_secs: Some(d.recovery_cooldown_secs),
            action_timeout_secs: Some(d.action_timeout_secs),
        }
    }
}

/// A resolved source is retained across reloads, even if its file disappears.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigSource {
    File(std::path::PathBuf),
    Defaults,
}

impl ConfigSource {
    pub fn resolve(explicit: Option<&std::path::Path>) -> Result<Self> {
        if let Some(path) = explicit {
            return Ok(Self::File(std::path::absolute(path)?));
        }
        if let Some(path) = std::env::var_os("CHECK_VPN_CONFIG") {
            return Ok(Self::File(std::path::absolute(std::path::PathBuf::from(
                path,
            ))?));
        }
        let local = std::env::current_dir()
            .context("failed to get current dir")?
            .join("check_vpn.xml");
        #[allow(unused_mut)]
        let mut candidates = vec![local];
        #[cfg(target_os = "macos")]
        if let Some(home) = std::env::var_os("HOME") {
            candidates.push(
                std::path::PathBuf::from(home).join("Library/Preferences/check_vpn/config.xml"),
            );
        }
        #[cfg(target_os = "linux")]
        {
            let base = std::env::var_os("XDG_CONFIG_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
                });
            if let Some(base) = base {
                candidates.push(base.join("check_vpn/config.xml"));
            }
        }
        #[cfg(unix)]
        candidates.push("/etc/check_vpn/config.xml".into());
        for path in candidates {
            if path
                .try_exists()
                .with_context(|| format!("checking configuration path {}", path.display()))?
            {
                return Ok(Self::File(std::path::absolute(path)?));
            }
        }
        Ok(Self::Defaults)
    }

    pub fn load(&self) -> Result<Config> {
        match self {
            Self::File(p) => Config::load_from_path(p),
            Self::Defaults => Ok(Config::default()),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        ConfigSource::resolve(None)?.load()
    }
    pub fn load_from_path(path: &std::path::Path) -> Result<Self> {
        crate::xml_io::read_xml(path).context("failed to read config via xml_io")
    }
}

impl EffectiveConfig {
    /// Normalize legacy convenience fields into one ordered provider list.
    pub fn provider_specs(&self) -> Vec<ProviderSpec> {
        let mut specs = Vec::new();
        if let Some(url) = &self.custom_json_server {
            specs.push(ProviderSpec {
                url: url.clone(),
                key: self.custom_json_key.clone(),
            });
        }
        specs.extend(self.providers.clone());
        specs.extend(self.provider_urls.iter().map(|url| ProviderSpec {
            url: url.clone(),
            key: None,
        }));
        specs
    }
    pub fn matches_normal_isp(&self, isp: &str) -> bool {
        isp == self.isp_to_check || self.isp_aliases.iter().any(|alias| alias == isp)
    }
}
