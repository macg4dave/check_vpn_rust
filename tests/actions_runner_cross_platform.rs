use check_vpn::actions::runner::{ActionRunner, RealActionRunner};

#[test]
fn runner_handles_dry_run_on_default_runner() {
    // Default runner (platform-dependent) should always succeed with dry-run
    let r = RealActionRunner::default();
    assert!(r.execute(&check_vpn::Action::Reboot, true).is_ok());
    assert!(r
        .execute(&check_vpn::Action::RestartUnit("x.service".into()), true)
        .is_ok());
    assert!(r
        .execute(&check_vpn::Action::Command("echo hi".into()), true)
        .is_ok());
}
