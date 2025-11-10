use std::net::TcpListener;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;

use check_vpn::config::EffectiveConfig;
use check_vpn::app::perform_check;

#[test]
fn perform_check_runs_action_when_isp_matches() {
    // start a listener so networking::is_online_with_ports can connect
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();

    // spawn acceptor so the listener is ready
    let _h = thread::spawn(move || {
        // accept one connection and then exit
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
        enable_ip_api: true,
        enable_ifconfig_co: true,
        provider_urls: vec![],
        custom_json_server: None,
        custom_json_key: None,
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
    // start a listener so networking::is_online_with_ports can connect
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();

    // spawn acceptor so the listener is ready
    let _h = thread::spawn(move || {
        // accept one connection and then exit
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
        enable_ip_api: true,
        enable_ifconfig_co: true,
        provider_urls: vec![],
        custom_json_server: None,
        custom_json_key: None,
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
