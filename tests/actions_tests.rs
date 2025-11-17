use check_vpn::actions::{parse_action, Action};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use check_vpn::config::EffectiveConfig;

fn make_eff(isp_to_check: &str) -> EffectiveConfig {
    EffectiveConfig {
        interval: 60,
        isp_to_check: isp_to_check.to_string(),
        action_type: "command".to_string(),
        action_arg: "echo hi".to_string(),
        dry_run: false,
        connectivity_endpoints: vec!["8.8.8.8".to_string()],
        connectivity_ports: vec![check_vpn::networking::DEFAULT_PORTS[0]],
        connectivity_timeout_secs: check_vpn::networking::DEFAULT_TIMEOUT_SECS,
        connectivity_retries: check_vpn::networking::DEFAULT_RETRIES,
        run_once: false,
        exit_on_error: false,
    }
}

#[test]
fn parse_reboot() {
    let a = parse_action("reboot", "");
    match a {
        Action::Reboot => {}
        _ => panic!("expected Reboot variant"),
    }
}

#[test]
fn parse_restart_unit() {
    let unit = "openvpn.service";
    let a = parse_action("restart-unit", unit);
    match a {
        Action::RestartUnit(s) => assert_eq!(s, unit.to_string()),
        _ => panic!("expected RestartUnit variant"),
    }
}

#[test]
fn parse_command() {
    let cmd = "/usr/bin/true";
    let a = parse_action("command", cmd);
    match a {
        Action::Command(s) => assert_eq!(s, cmd.to_string()),
        _ => panic!("expected Command variant"),
    }
}

#[test]
fn parse_unknown_fallbacks_to_command() {
    let arg = "something";
    let a = parse_action("unknown-thing", arg);
    match a {
        Action::Command(s) => assert_eq!(s, arg.to_string()),
        _ => panic!("expected Command fallback"),
    }
}

#[test]
fn perform_check_triggers_action_when_isp_matches() {
    let eff = make_eff("TARGET_ISP");

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let get_isp = || -> Result<String, anyhow::Error> { Ok("TARGET_ISP".to_string()) };
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        called_clone.store(true, Ordering::SeqCst);
    };

    let res = check_vpn::app::perform_check(&eff, get_isp, run_action);
    assert!(res.is_ok());
    assert!(
        called.load(Ordering::SeqCst),
        "expected action to be triggered"
    );
}

#[test]
fn perform_check_does_not_trigger_action_when_isp_differs() {
    let eff = make_eff("TARGET_ISP");

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let get_isp = || -> Result<String, anyhow::Error> { Ok("OTHER_ISP".to_string()) };
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        called_clone.store(true, Ordering::SeqCst);
    };

    let res = check_vpn::app::perform_check(&eff, get_isp, run_action);
    assert!(res.is_ok());
    assert!(
        !called.load(Ordering::SeqCst),
        "expected no action when ISP differs"
    );
}

#[test]
fn perform_check_handles_get_isp_error_without_exiting_when_nonfatal() {
    let mut eff = make_eff("TARGET_ISP");
    eff.run_once = false;
    eff.exit_on_error = false;

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let get_isp = || -> Result<String, anyhow::Error> { Err(anyhow::anyhow!("network fail")) };
    let run_action = move |_a: &check_vpn::actions::Action, _dry: bool| {
        called_clone.store(true, Ordering::SeqCst);
    };

    // Should return Ok(()) and not call run_action
    let res = check_vpn::app::perform_check(&eff, get_isp, run_action);
    assert!(res.is_ok());
    assert!(
        !called.load(Ordering::SeqCst),
        "action should not be called on get_isp error when non-fatal"
    );
}
