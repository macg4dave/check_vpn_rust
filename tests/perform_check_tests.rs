use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use check_vpn::config::Config;
use check_vpn::cli::Args;

#[test]
fn perform_check_runs_action_when_isp_matches() {
    // Build effective config with a specific isp_to_check
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: false,
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
    let run_action = move |_a: &check_vpn::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Ok("ISP-TEST".to_string()) };

    check_vpn::app::perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(ran.load(Ordering::SeqCst), "expected action to run when ISP matches");
}

#[test]
fn perform_check_does_not_run_action_when_isp_differs() {
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: false,
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
    let run_action = move |_a: &check_vpn::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Ok("OTHER-ISP".to_string()) };

    check_vpn::app::perform_check(&eff, get_isp, run_action).expect("perform_check");

    assert!(!ran.load(Ordering::SeqCst), "did not expect action to run when ISP differs");
}

#[test]
fn perform_check_handles_get_isp_error_gracefully() {
    let args = Args {
        interval: None,
        isp_to_check: Some("ISP-TEST".to_string()),
        vpn_lost_action_type: None,
        vpn_lost_action_arg: None,
        dry_run: false,
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
    let run_action = move |_a: &check_vpn::Action, _dry: bool| {
        ran_clone.store(true, Ordering::SeqCst);
    };

    let get_isp = || -> Result<String, anyhow::Error> { Err(anyhow::anyhow!("no isp")) };

    // Should not panic/exit; perform_check logs and returns Ok
    let res = check_vpn::app::perform_check(&eff, get_isp, run_action);
    assert!(res.is_ok(), "perform_check should return Ok on get_isp error in non-fatal mode");
    assert!(!ran.load(Ordering::SeqCst), "action should not run when get_isp fails");
}
