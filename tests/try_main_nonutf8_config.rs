#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

#[test]
#[cfg(unix)]
fn try_run_with_args_handles_non_utf8_config_paths() {
    use std::ffi::OsString;
    use std::fs;
    use std::path::PathBuf;

    // Create a filename with invalid UTF-8 bytes
    let fname = OsString::from_vec(b"cfg-\xff-not-utf8.xml".to_vec());
    let td = tempfile::tempdir().expect("tempdir");
    let mut path = td.path().to_path_buf();
    path.push(&fname);

    // Write non-XML contents so load_from_path will return an error
    // Some OS/filesystems (notably macOS) may refuse to create filenames
    // containing certain invalid byte sequences. If creating the file fails
    // with EILSEQ (illegal byte sequence) we skip the test instead of failing.
    match fs::write(&path, "Not XML") {
        Ok(()) => {}
        Err(e) => {
            if e.raw_os_error() == Some(92) {
                eprintln!(
                    "skipping test: platform refused to create non-UTF8 filename: {}",
                    e
                );
                return;
            }
            panic!("write malformed file: {}", e);
        }
    }

    // Build Args with the testing config path (non-UTF8 path as PathBuf)
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
        config: Some(PathBuf::from(&path)),
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
    assert!(matches!(
        res,
        Err(check_vpn::error_types::AppError::ConfigLoadError(_))
    ));
}
