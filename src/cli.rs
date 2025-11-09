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
}

impl Args {
    pub fn parse_args() -> Self {
        Args::parse()
    }
}
