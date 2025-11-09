use log::error;

fn main() {
    // Parse CLI and load config (CLI takes precedence in merging within app)
    let args = check_vpn::cli::Args::parse_args();

    // Initialize logging early using CLI verbosity so app.run can log at the
    // requested level. This respects RUST_LOG if already set.
    check_vpn::logging::init_with_verbosity(args.verbose);
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
