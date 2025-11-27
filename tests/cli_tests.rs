use check_vpn::cli::Args;
use clap::Parser;

#[test]
fn parse_args_connectivity_and_flags() {
    // Simulate CLI: --connectivity-endpoint 1.2.3.4 --connectivity-ports 443,53 --exit-on-error
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
