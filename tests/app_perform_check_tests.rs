use std::net::TcpListener;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

use check_vpn::config::{Config, EffectiveConfig};
use check_vpn::cli::Args;
use check_vpn::app::perform_check;

/// Test helper to create a live TCP listener and return the port
fn create_test_listener() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    (listener, port)
}

#[test]
fn perform_check_runs_action_when_isp_matches() {
    // Start a listener so networking::is_online_with_ports can connect
    let (listener, port) = create_test_listener();

    // Spawn acceptor so the listener is ready
    let _h = thread::spawn(move || {
        // Accept one connection and then exit
        let _ = listener.accept();
    });

    let eff = EffectiveConfig {
        interval: 60,
        isp_to_check: "ISP A".to_string(),
        action_type: "reboot".to_string(),
        action_arg: "".to_string(),
        dry_run: true,
        connectivity_endpoints: vec!["127.0.0.1".to_string()],
        connectivity_ports: vec![port],
        connectivity_timeout_secs: 1,
        connectivity_retries: 1,
        run_once: true,
        exit_on_error: false,
    };

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let get_isp = || Ok("ISP A".to_string());
    let run_action = move |_a: &check_vpn::actions::Action, _d: bool| {
        called_clone.store(true, Ordering::SeqCst);
    };

    perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(called.load(Ordering::SeqCst));
}

#[test]
fn perform_check_does_not_run_action_when_isp_differs() {
    // Start a listener so networking::is_online_with_ports can connect
    let (listener, port) = create_test_listener();

    // Spawn acceptor so the listener is ready
    let _h = thread::spawn(move || {
        // Accept one connection and then exit
        let _ = listener.accept();
    });

    let eff = EffectiveConfig {
        interval: 60,
        isp_to_check: "ISP A".to_string(),
        action_type: "reboot".to_string(),
        action_arg: "".to_string(),
        dry_run: true,
        connectivity_endpoints: vec!["127.0.0.1".to_string()],
        connectivity_ports: vec![port],
        connectivity_timeout_secs: 1,
        connectivity_retries: 1,
        run_once: true,
        exit_on_error: false,
    };

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let get_isp = || Ok("Other ISP".to_string());
    let run_action = move |_a: &check_vpn::actions::Action, _d: bool| {
        called_clone.store(true, Ordering::SeqCst);
    };

    perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(!called.load(Ordering::SeqCst));
}

#[test]
fn perform_check_with_args_config_runs_action_when_isp_matches() {
    // Test using Args + Config::merge_with_args approach
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: true,
        connectivity_endpoints: None,
        connectivity_ports: None,
        connectivity_timeout_secs: None,
        connectivity_retries: None,
        run_once: false,
        verbose: 0,
        enable_metrics: false,
        metrics_addr: "127.0.0.1:0".to_string(),
        exit_on_error: false,
    };

    let eff = Config::default().merge_with_args(&args);

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Ok("ISP-TEST".to_string()) };

    perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(ran.load(Ordering::SeqCst), "expected action to run when ISP matches");
}

#[test]
fn perform_check_with_args_config_does_not_run_action_when_isp_differs() {
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: true,
        connectivity_endpoints: None,
        connectivity_ports: None,
        connectivity_timeout_secs: None,
        connectivity_retries: None,
        run_once: false,
        verbose: 0,
        enable_metrics: false,
        metrics_addr: "127.0.0.1:0".to_string(),
        exit_on_error: false,
    };

    let eff = Config::default().merge_with_args(&args);

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Ok("OTHER-ISP".to_string()) };

    perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(!ran.load(Ordering::SeqCst), "did not expect action to run when ISP differs");
}

#[test]
fn perform_check_handles_get_isp_error_gracefully() {
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: true,
        connectivity_endpoints: None,
        connectivity_ports: None,
        connectivity_timeout_secs: None,
        connectivity_retries: None,
        run_once: false,
        verbose: 0,
        enable_metrics: false,
        metrics_addr: "127.0.0.1:0".to_string(),
        exit_on_error: false,
    };

    let eff = Config::default().merge_with_args(&args);

    let ran = Arc::new(AtomicBool::new(false));
    let ran_clone = ran.clone();
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Err(anyhow::anyhow!("no isp")) };

    // Should not panic/exit; perform_check logs and returns Ok
    let res = perform_check(&eff, get_isp, run_action);
    assert!(res.is_ok(), "perform_check should return Ok on get_isp error in non-fatal mode");
    assert!(!ran.load(Ordering::SeqCst), "action should not run when get_isp fails");
}