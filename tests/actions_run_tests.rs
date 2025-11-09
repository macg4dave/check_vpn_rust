use check_vpn::actions::{Action, run_action};

#[test]
fn run_action_dry_run_variants_do_not_panic() {
    // All variants should be callable in dry-run mode without attempting external or D-Bus actions.
    let a1 = Action::Reboot;
    run_action(&a1, true);

    let a2 = Action::RestartUnit("example.service".to_string());
    run_action(&a2, true);

    let a3 = Action::Command("/bin/true".to_string());
    run_action(&a3, true);
}
