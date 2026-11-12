use check_vpn::actions::{runner::run_process, Action};
use check_vpn::app::{check_identity, perform_check, CheckOutcome};
use check_vpn::config::{Config, ConfigSource, EffectiveConfig, ProviderSpec};
use check_vpn::providers::{self, VpnInfoProvider};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

#[test]
fn packaged_and_generated_configuration_roundtrip_with_lists() {
    let cfg = Config::load_from_path(std::path::Path::new("contrib/config.xml")).unwrap();
    assert_eq!(cfg.connectivity_endpoints.as_ref().unwrap().len(), 2);
    assert_eq!(cfg.connectivity_ports.as_ref().unwrap(), &[443, 53]);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("generated.xml");
    let mut cfg = cfg;
    cfg.isp_aliases = Some(vec!["Normal ASN Org".into(), "Normal ISP".into()]);
    cfg.providers = Some(vec![ProviderSpec {
        url: "https://example.com/json".into(),
        key: Some("org".into()),
    }]);
    check_vpn::xml_io::write_xml(&cfg, &path).unwrap();
    assert_eq!(Config::load_from_path(&path).unwrap(), cfg);
    check_vpn::init_wizard::set_no_prompt_for_tests(true);
    check_vpn::init_wizard::run_init(Some(path.to_str().unwrap()), true).unwrap();
    let generated = Config::load_from_path(&path).unwrap();
    let eff = generated.merge_with_args(&Default::default());
    Config::validate_values(&eff).unwrap();
    assert_eq!(eff.action_type, "log");
    assert_eq!(
        eff.connectivity_ports,
        EffectiveConfig::default().connectivity_ports
    );
}

#[test]
fn frozen_source_errors_when_file_disappears_and_defaults_stay_defaults() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.xml");
    std::fs::write(&path, "<config><interval>7</interval></config>").unwrap();
    let source = ConfigSource::resolve(Some(&path)).unwrap();
    assert_eq!(source.load().unwrap().interval, Some(7));
    std::fs::remove_file(path).unwrap();
    assert!(source.load().is_err());
    assert_eq!(ConfigSource::Defaults.load().unwrap(), Config::default());
}

#[test]
fn exact_normal_isp_and_aliases_trigger_recovery_even_when_probes_are_blocked() {
    let eff = EffectiveConfig {
        isp_to_check: "Normal ISP".into(),
        isp_aliases: vec!["Normal ASN Org".into()],
        connectivity_endpoints: vec!["127.0.0.1:1".into()],
        ..Default::default()
    };
    for isp in ["Normal ISP", "Normal ASN Org"] {
        let outcome =
            check_identity(&eff, &Action::Log, true, || Ok(isp.into()), |_, _| Ok(())).unwrap();
        assert_eq!(outcome, CheckOutcome::RecoveryAttempted(isp.into()));
    }
    for isp in ["Unknown VPN ISP", "normal isp", "Normal ISP "] {
        let outcome = check_identity(
            &eff,
            &Action::Log,
            true,
            || Ok(isp.into()),
            |_, _| panic!("unexpected action"),
        )
        .unwrap();
        assert_eq!(outcome, CheckOutcome::VpnActive(isp.into()));
    }
}

#[test]
fn daemon_action_and_lookup_errors_share_the_fatal_policy() {
    for fatal in [false, true] {
        let eff = EffectiveConfig {
            exit_on_error: fatal,
            ..Default::default()
        };
        assert_eq!(
            perform_check(
                &eff,
                || Ok(eff.isp_to_check.clone()),
                |_, _| anyhow::bail!("action failed")
            )
            .is_err(),
            fatal
        );
        assert_eq!(
            perform_check(
                &eff,
                || anyhow::bail!("lookup failed"),
                |_, _| panic!("unexpected action")
            )
            .is_err(),
            fatal
        );
    }
    let eff = EffectiveConfig {
        run_once: true,
        ..Default::default()
    };
    assert!(perform_check(
        &eff,
        || Ok(eff.isp_to_check.clone()),
        |_, _| anyhow::bail!("action failed")
    )
    .is_err());
}

#[test]
fn cooldown_defers_recovery_but_still_reports_vpn_loss() {
    let eff = EffectiveConfig::default();
    assert_eq!(
        check_identity(
            &eff,
            &Action::Log,
            false,
            || Ok(eff.isp_to_check.clone()),
            |_, _| panic!("cooldown must suppress recovery")
        )
        .unwrap(),
        CheckOutcome::RecoveryDeferred(eff.isp_to_check)
    );
}

#[test]
fn blank_identity_falls_back_and_does_not_count_as_active_vpn() {
    struct Fixed(&'static str);
    impl VpnInfoProvider for Fixed {
        fn name(&self) -> &str {
            "fixture"
        }
        fn query(&self) -> anyhow::Result<providers::VpnIdentity> {
            Ok(providers::VpnIdentity { isp: self.0.into() })
        }
    }
    let chain: Vec<Box<dyn VpnInfoProvider>> =
        vec![Box::new(Fixed("  ")), Box::new(Fixed("Normal ISP"))];
    assert_eq!(
        providers::query_first_success(&chain).unwrap().isp,
        "Normal ISP"
    );
    assert!(check_identity(
        &EffectiveConfig::default(),
        &Action::Log,
        true,
        || Ok(" ".into()),
        |_, _| panic!()
    )
    .is_err());
}

#[test]
fn invalid_runtime_configuration_is_rejected_before_monitoring() {
    let defaults = EffectiveConfig::default();
    for url in [
        "",
        "not a URL",
        "file:///tmp/identity",
        "https://user:password@example.com",
    ] {
        let eff = EffectiveConfig {
            custom_json_server: Some(url.into()),
            ..defaults.clone()
        };
        assert!(Config::validate_values(&eff).is_err(), "accepted {url}");
    }
    for command in ["echo \"unclosed", "\"\"", "echo hi; echo bye"] {
        let eff = EffectiveConfig {
            action_type: "command".into(),
            action_arg: command.into(),
            ..defaults.clone()
        };
        assert!(Config::validate_values(&eff).is_err(), "accepted {command}");
    }
    let eff = EffectiveConfig {
        custom_json_server: Some("https://example.com".into()),
        custom_json_key: Some(" ".into()),
        ..defaults.clone()
    };
    assert!(Config::validate_values(&eff).is_err());
    let eff = EffectiveConfig {
        action_timeout_secs: 0,
        ..defaults
    };
    assert!(Config::validate_values(&eff).is_err());
}

#[test]
fn dns_failure_does_not_hide_reachable_later_endpoint() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let good = listener.local_addr().unwrap().to_string();
    assert!(check_vpn::networking::is_online_with_retries(
        &["invalid host:80", &good],
        1,
        &[80],
        1
    )
    .unwrap());
}

#[test]
fn cancellation_prevents_provider_queries_and_retry_waits() {
    let cancel = AtomicBool::new(true);
    let start = Instant::now();
    assert!(providers::query_first_success_with_cancel(&[], &cancel).is_err());
    assert!(providers::http::wait(Duration::from_secs(60), Some(&cancel)).is_err());
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[test]
fn provider_deadline_caps_long_retry_after() {
    use httpmock::{Method::GET, MockServer};
    let server = MockServer::start();
    server.mock(|when, then| {
        when.method(GET);
        then.status(429).header("Retry-After", "60");
    });
    let start = Instant::now();
    let result = providers::http::get_json(
        &providers::http::client().unwrap(),
        &server.url("/json"),
        2,
        None,
        Duration::from_millis(150),
    );
    assert!(result.is_err());
    assert!(start.elapsed() < Duration::from_secs(2));
}

#[test]
fn cancellation_interrupts_recovery_process() {
    use std::sync::{atomic::Ordering, Arc};
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancel);
    let thread = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(150));
        signal.store(true, Ordering::Relaxed);
    });
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "subprocess_child_fixture"])
        .env("CHECK_VPN_SLEEP_FIXTURE", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let start = Instant::now();
    let result = run_process(&mut command, Duration::from_secs(30), &cancel);
    thread.join().unwrap();
    assert!(result.unwrap_err().to_string().contains("cancelled"));
    assert!(start.elapsed() < Duration::from_secs(5));
}

// Spawn this test executable as a harmless, portable long-running child.
#[test]
fn subprocess_child_fixture() {
    if std::env::var_os("CHECK_VPN_SLEEP_FIXTURE").is_some() {
        std::thread::sleep(Duration::from_secs(120));
    }
}

#[test]
fn recovery_process_timeout_terminates_owned_child() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "subprocess_child_fixture"])
        .env("CHECK_VPN_SLEEP_FIXTURE", "1")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let start = Instant::now();
    let error = run_process(
        &mut command,
        Duration::from_millis(150),
        &AtomicBool::new(false),
    )
    .unwrap_err();
    assert!(error.to_string().contains("timed out"));
    assert!(start.elapsed() < Duration::from_secs(5));
}

#[test]
fn daemon_continues_after_failed_action_and_respects_cooldown() {
    use httpmock::{Method::GET, MockServer};
    struct ChildGuard(std::process::Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("daemon.log");
    let log = std::fs::File::create(&log_path).unwrap();
    let config = dir.path().join("config.xml");
    std::fs::write(&config, "<config><interval>1</interval><isp_to_check>Normal ISP</isp_to_check><vpn_lost_action_type>command</vpn_lost_action_type><vpn_lost_action_arg>check_vpn_nonexistent_recovery_fixture</vpn_lost_action_arg></config>").unwrap();
    let server = MockServer::start();
    let mock = server.mock(|when, then| {
        when.method(GET);
        then.status(200).body(r#"{"isp":"Normal ISP"}"#);
    });
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_check_vpn"))
            .args([
                "--config",
                config.to_str().unwrap(),
                "--disable-ip-api",
                "--disable-ifconfig-co",
                "--provider-url",
                &server.url("/json"),
            ])
            .env("RUST_LOG", "check_vpn=info")
            .env("RUST_LOG_STYLE", "never")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    while mock.calls() < 3 && start.elapsed() < Duration::from_secs(10) {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "daemon stopped after recovery error"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        mock.calls() >= 3,
        "daemon did not continue querying providers"
    );
    assert!(child.0.try_wait().unwrap().is_none());
    let logs = std::fs::read_to_string(log_path).unwrap();
    assert_eq!(
        logs.matches("continuing monitoring").count(),
        1,
        "action retried during cooldown: {logs}"
    );
}

#[cfg(unix)]
#[test]
fn valid_non_utf8_configuration_path_loads_successfully() {
    use std::os::unix::ffi::OsStringExt;
    let dir = tempfile::tempdir().unwrap();
    let path = dir
        .path()
        .join(std::ffi::OsString::from_vec(b"cfg-\xff.xml".to_vec()));
    std::fs::write(&path, "<config><interval>11</interval></config>").unwrap();
    assert_eq!(Config::load_from_path(&path).unwrap().interval, Some(11));
}
