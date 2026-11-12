use std::path::PathBuf;

#[test]
fn try_run_with_args_config_load_error_maps_to_exit_code() {
    // Create a malformed temporary file so loading it triggers a ConfigLoadError.
    let td = tempfile::tempdir().expect("tempdir");
    let mut p = td.path().to_path_buf();
    p.push("bad_config.xml");
    std::fs::write(&p, "Not XML").expect("write malformed xml");

    // Build Args with a regular (UTF-8) path to the malformed file.
    let args = check_vpn::cli::Args {
        interval: None,
        isp_to_check: None,
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: false,
        connectivity_endpoints: None,
        connectivity_ports: None,
        connectivity_timeout_secs: None,
        connectivity_retries: None,
        run_once: false,
        verbose: 0,
        config: Some(PathBuf::from(&p)),
        exit_on_error: false,
        strict_config: false,
        disable_ip_api: false,
        disable_ifconfig_co: false,
        provider_urls: None,
        custom_json_server: None,
        custom_json_key: None,
        init: false,
        init_output: None,
        init_no_fetch: false,
        generate_completions: None,
        completions_output: None,
        isp_aliases: None,
        recovery_cooldown_secs: None,
        action_timeout_secs: None,
    };

    let res = check_vpn::try_run_with_args(args);

    // Expect ConfigLoadError
    let err = match res {
        Err(e @ check_vpn::error_types::AppError::ConfigLoadError(_)) => e,
        Err(e) => panic!("expected ConfigLoadError, got different AppError: {:?}", e),
        Ok(()) => panic!("expected error, but got Ok(())"),
    };

    let ec = check_vpn::error_types::exit_code_for_error(&err);
    assert_eq!(
        ec,
        std::process::ExitCode::from(check_vpn::config::EXIT_INVALID_CONFIG as u8)
    );
}
