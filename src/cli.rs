use clap::Parser;

/// Command-line arguments for check_vpn
#[derive(Parser, Debug)]
#[command(author, version, about = "VPN checker (Rust port)")]
pub struct Args {
    /// Seconds between checks (overrides config)
    #[arg(short, long)]
    pub interval: Option<u64>,

    /// ISP string that indicates VPN is lost (overrides config)
    #[arg(short = 'i', long)]
    pub isp_to_check: Option<String>,

    /// Action type to run when VPN is lost. One of: reboot, restart-unit, command (overrides config)
    #[arg(short = 't', long)]
    pub vpn_lost_action_type: Option<String>,

    /// Argument for the action. For `restart-unit` this is the systemd unit name. For `command` it's the command string. (overrides config)
    #[arg(short = 'a', long)]
    pub vpn_lost_action_arg: Option<String>,

    /// Do not actually run actions, just log what would happen (overrides config)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Connectivity endpoints to check (can be repeated). Example: --connectivity-endpoint 8.8.8.8 --connectivity-endpoint google.com
    #[arg(long = "connectivity-endpoint", num_args = 1.., value_delimiter = ',')]
    pub connectivity_endpoints: Option<Vec<String>>,

    /// Connectivity ports to try when endpoint has no port (comma-separated or repeated). Example: --connectivity-ports 443,53
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

    /// Exit with non-zero codes on errors even in long-running mode (useful for health checks)
    #[arg(long = "exit-on-error", action = clap::ArgAction::SetTrue)]
    pub exit_on_error: bool,
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
