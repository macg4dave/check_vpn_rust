use check_vpn::actions::{parse_action, Action};

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
