use log::{error, warn};

pub mod runner;
use runner::{ActionRunner, RealActionRunner};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Reboot,
    RestartUnit(String),
    Command(String), // fallback: executes an external command
}

/// Parse an action type and argument into an `Action` enum.
///
/// Accepted `action_type` values:
/// - "reboot" -> `Action::Reboot`
/// - "restart-unit" -> `Action::RestartUnit(arg)`
/// - "command" -> `Action::Command(arg)`
///   Any other value will be logged as a warning and treated as a `Command` fallback.
pub fn parse_action(action_type: &str, arg: &str) -> Action {
    match action_type {
        "reboot" => Action::Reboot,
        "restart-unit" => Action::RestartUnit(arg.to_string()),
        "command" => Action::Command(arg.to_string()),
        other => {
            warn!(
                "Unknown action type '{}', falling back to command with given arg",
                other
            );
            Action::Command(arg.to_string())
        }
    }
}

/// Convenience function that mirrors the previous public API: execute the action and swallow errors.
///
/// This preserves compatibility with existing call sites which expect `run_action(&Action, bool)`.
pub fn run_action(action: &Action, dry_run: bool) {
    let runner = RealActionRunner::new();
    if let Err(e) = runner.execute(action, dry_run) {
        error!("Action execution failed: {}", e);
    }
}

// Small unit tests that exercise parsing and the dry-run paths of the real runner.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_action_known_types() {
        assert_eq!(parse_action("reboot", ""), Action::Reboot);
        assert_eq!(
            parse_action("restart-unit", "ssh.service"),
            Action::RestartUnit("ssh.service".to_string())
        );
        assert_eq!(
            parse_action("command", "echo hi"),
            Action::Command("echo hi".to_string())
        );
    }

    #[test]
    fn parse_action_unknown_falls_back_to_command() {
        let a = parse_action("weird", "something");
        assert_eq!(a, Action::Command("something".to_string()));
    }

    #[test]
    fn real_runner_dry_run_variants_ok() {
        let runner = RealActionRunner::new();
        // Dry-run should not attempt system calls and should return Ok(())
        assert!(runner.execute(&Action::Reboot, true).is_ok());
        assert!(runner
            .execute(&Action::RestartUnit("unit.service".into()), true)
            .is_ok());
        assert!(runner
            .execute(&Action::Command("echo hi".into()), true)
            .is_ok());
    }
}
