//! CLI argument definitions and helpers for the `check_vpn` crate.
//!
//! This module holds the public `Args` type (re-exported as `crate::cli::Args`).
//! The original implementation lived in `src/cli.rs`; it was moved here to allow
//! future submodules (parsing helpers, tests) while keeping the crate root tidy.

use clap::Parser;

/// Command-line arguments for `check_vpn`.
///
/// This struct is intentionally lean: it only represents the raw CLI values as
/// parsed by `clap`. Higher-level merging with XML config and validation is
/// performed in `crate::config` so the responsibilities remain separated.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about = "VPN checker (Rust port)")]
pub struct Args {
    /// Seconds between checks (overrides config)
    ///
    /// Long flag is `--interval`. Short flag was removed to avoid a collision
    /// with other short options in the original tool.
    #[arg(long = "interval")]
    pub interval: Option<u64>,

    /// ISP string that indicates VPN is lost (overrides config)
    #[arg(short = 'i', long)]
    pub isp_to_check: Option<String>,

    /// Action type to run when VPN is lost. One of: reboot, restart-unit, command
    #[arg(short = 't', long)]
    pub vpn_lost_action_type: Option<String>,

    /// Argument for the action. For `restart-unit` this is the systemd unit
    /// name. For `command` it's the command string. (overrides config)
    #[arg(short = 'a', long)]
    pub vpn_lost_action_arg: Option<String>,

    /// Do not actually run actions, just log what would happen (overrides config)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Connectivity endpoints to check (can be repeated). Example:
    /// `--connectivity-endpoint 8.8.8.8 --connectivity-endpoint google.com`.
    ///
    /// Comma-delimited values are also supported when passed in a single flag.
    #[arg(long = "connectivity-endpoint", num_args = 1.., value_delimiter = ',')]
    pub connectivity_endpoints: Option<Vec<String>>,

    /// Connectivity ports to try when endpoint has no port (comma-separated or repeated).
    /// Example: `--connectivity-ports 443,53`
    #[arg(long = "connectivity-ports", num_args = 1.., value_delimiter = ',', value_parser = clap::value_parser!(u16))]
    pub connectivity_ports: Option<Vec<u16>>,

    /// Timeout in seconds for connectivity checks (overrides config)
    #[arg(long = "connectivity-timeout")]
    pub connectivity_timeout_secs: Option<u64>,

    /// Number of retries for connectivity checks (overrides config)
    #[arg(long = "connectivity-retries")]
    pub connectivity_retries: Option<usize>,

    /// Run only a single iteration and exit (useful for testing)
    #[arg(long = "run-once", action = clap::ArgAction::SetTrue)]
    pub run_once: bool,

    /// Increase logging verbosity. May be repeated (-v, -vv) to reach debug/trace levels.
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Enable a simple HTTP health/metrics endpoint (useful for container deployments)
    #[arg(long = "enable-metrics", action = clap::ArgAction::SetTrue)]
    pub enable_metrics: bool,

    /// Address to bind the health/metrics endpoint (default: 0.0.0.0:9090)
    #[arg(long = "metrics-addr", default_value = "0.0.0.0:9090")]
    pub metrics_addr: String,

    /// Exit with non-zero codes on errors even in long-running mode (useful for health checks)
    #[arg(long = "exit-on-error", action = clap::ArgAction::SetTrue)]
    pub exit_on_error: bool,

    /// Disable the ip-api.com provider (enabled by default). Use when rate-limited.
    #[arg(long = "disable-ip-api", action = clap::ArgAction::SetTrue)]
    pub disable_ip_api: bool,

    /// Disable the ifconfig.co provider (enabled by default).
    #[arg(long = "disable-ifconfig-co", action = clap::ArgAction::SetTrue)]
    pub disable_ifconfig_co: bool,

    /// Custom provider JSON endpoints returning at least an `isp` or `asn_org` field.
    /// May be repeated or comma-delimited. When provided, these are queried first in
    /// the order given before built-in providers.
    #[arg(long = "provider-url", num_args = 1.., value_delimiter = ',')]
    pub provider_urls: Option<Vec<String>>,

    /// Single custom JSON server to query (convenience for one-off custom source)
    #[arg(long = "custom-json-server")]
    pub custom_json_server: Option<String>,

    /// Specific JSON key to extract from the custom server (e.g. asn, org, isp)
    #[arg(long = "custom-json-key")]
    pub custom_json_key: Option<String>,

    /// Wizard mode: generate a config file interactively and exit
    #[arg(long = "init", action = clap::ArgAction::SetTrue)]
    pub init: bool,
    /// Output path for generated config (defaults to ./check_vpn.xml)
    #[arg(long = "init-output")]
    pub init_output: Option<String>,
    /// Skip fetching current ISP during init (leave placeholder instead)
    #[arg(long = "init-no-fetch", action = clap::ArgAction::SetTrue)]
    pub init_no_fetch: bool,
}

impl Args {
    /// Parse arguments from the program args (wrapper around `clap`'s `parse`).
    ///
    /// Keeping this thin wrapper preserves the original crate API and keeps
    /// the call site concise: `let args = crate::cli::Args::parse_args();`.
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

// Unit tests for CLI parsing. These are intentionally small and focused on
// ensuring that argument parsing and attribute wiring behave as expected.
#[cfg(test)]
mod tests {
    use super::Args;
    use clap::Parser;

    #[test]
    fn parse_args_connectivity_and_flags() {
        let argv = vec![
            "check_vpn",
            "--connectivity-endpoint",
            "1.2.3.4",
            "--connectivity-ports",
            "443,53",
            "--exit-on-error",
            "--isp-to-check",
            "TestISP",
            "--interval",
            "120",
        ];

        let args = Args::parse_from(argv);
        assert!(args.connectivity_endpoints.is_some());
        let eps = args.connectivity_endpoints.unwrap();
        assert_eq!(eps, vec!["1.2.3.4".to_string()]);

        assert!(args.connectivity_ports.is_some());
        let ports = args.connectivity_ports.unwrap();
        assert_eq!(ports, vec![443u16, 53u16]);

        assert!(args.exit_on_error);
        assert_eq!(args.isp_to_check.unwrap(), "TestISP");
        assert_eq!(args.interval.unwrap(), 120);
    }

    #[test]
    fn defaults_and_metrics_addr() {
        let argv = vec!["check_vpn"];
        let args = Args::parse_from(argv);
        // default metric address provided by clap default_value
        assert_eq!(args.metrics_addr, "0.0.0.0:9090");
        // defaults for booleans are false
        assert!(!args.dry_run);
        assert!(!args.enable_metrics);
        assert!(!args.exit_on_error);
        // provider disables default false
        assert!(!args.disable_ip_api);
        assert!(!args.disable_ifconfig_co);
    }

    #[test]
    fn parse_provider_flags_and_urls() {
        let argv = vec![
            "check_vpn",
            "--disable-ip-api",
            "--provider-url",
            "http://example.test/json1,http://example.test/json2",
            "--custom-json-server",
            "http://single.example/json",
            "--custom-json-key",
            "asn",
            "--init",
            "--init-output",
            "./gen.xml",
            "--init-no-fetch",
        ];
        let args = Args::parse_from(argv);
        assert!(args.disable_ip_api);
        assert!(!args.disable_ifconfig_co);
        let urls = args.provider_urls.clone().unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "http://example.test/json1");
        assert_eq!(args.custom_json_server.as_ref().unwrap(), "http://single.example/json");
        assert_eq!(args.custom_json_key.as_ref().unwrap(), "asn");
        assert!(args.init);
        assert_eq!(args.init_output.as_ref().unwrap(), "./gen.xml");
        assert!(args.init_no_fetch);
    }
}
