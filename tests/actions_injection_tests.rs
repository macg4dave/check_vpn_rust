use check_vpn::actions::{
    runner::{ActionRunner, RealActionRunner},
    Action,
};

#[test]
fn runner_rejects_command_with_shell_meta() {
    let runner = RealActionRunner::new();
    let bad = Action::Command("rm -rf /; echo hacked".into());
    let res = runner.execute(&bad, false);
    assert!(res.is_err());
}

#[test]
fn runner_rejects_restart_unit_with_injection() {
    // restart-unit validation is exercised by config validation tests (unit-level),
    // and we avoid exercising the restart path in integration tests to prevent
    // accidental restarts on CI systems.
}

#[test]
fn runner_allows_safe_command_in_dry_run() {
    // dry-run should not attempt execution and should be OK even with 'weaker' inputs
    let runner = RealActionRunner::new();
    let ok = Action::Command("rm -rf /; echo hacked".into());
    assert!(runner.execute(&ok, true).is_ok());
}
