use log::error;

fn main() {
    // Initialize logging early so app.run can log.
    check_vpn::logging::init();

    // Parse CLI and load config (CLI takes precedence in merging within app)
    let args = check_vpn::cli::Args::parse_args();
    let cfg = match check_vpn::config::Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load config, using defaults: {}", e);
            check_vpn::config::Config::default()
        }
    };

    if let Err(e) = check_vpn::app::run(args, cfg) {
        error!("fatal error running app: {}", e);
        std::process::exit(1);
    }

}
