use log::warn;

pub mod runner;
use runner::{ActionRunner, RealActionRunner};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Log,
    Reboot,
    RestartUnit(String),
    Command(String), // fallback: executes an external command
}

/// Parse an action type and argument into an `Action` enum.
///
/// Accepted `action_type` values:
/// - "log" -> `Action::Log`
/// - "reboot" -> `Action::Reboot`
/// - "restart-unit" -> `Action::RestartUnit(arg)`
/// - "command" -> `Action::Command(arg)`
///   Any other value will be logged as a warning and treated as a `Command` fallback.
pub fn parse_action(action_type: &str, arg: &str) -> Action {
    match action_type {
        "log" => Action::Log,
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

/// Parse a command once, preserving quoted executable paths and arguments.
pub fn command_parts(command: &str) -> anyhow::Result<Vec<String>> {
    anyhow::ensure!(
        crate::config::validation::is_safe_command_string(command),
        "command contains forbidden shell meta-characters or is empty"
    );
    let parts = shell_words::split(command)?;
    anyhow::ensure!(
        parts.first().is_some_and(|part| !part.trim().is_empty()),
        "empty executable"
    );
    Ok(parts)
}

/// Validated action with command arguments prepared at configuration time.
pub struct PreparedAction {
    pub(crate) action: Action,
    pub(crate) command: Option<Vec<String>>,
}
impl PreparedAction {
    pub fn action(&self) -> &Action {
        &self.action
    }
    pub fn new(action: Action) -> anyhow::Result<Self> {
        let command = match &action {
            Action::Command(s) => Some(command_parts(s)?),
            _ => None,
        };
        if let Action::RestartUnit(unit) = &action {
            anyhow::ensure!(
                crate::config::validation::is_valid_unit_name(unit),
                "invalid systemd unit name"
            );
        }
        Ok(Self { action, command })
    }
}

pub fn run_action(action: &Action, dry_run: bool) -> anyhow::Result<()> {
    RealActionRunner::new().execute(action, dry_run)
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
