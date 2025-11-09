use anyhow::{Context, Result};
use log::{error, info};
use std::process::Command;
use zbus::blocking::Connection;

#[derive(Debug)]
pub enum Action {
    Reboot,
    RestartUnit(String),
    Command(String), // fallback: executes an external command
}

pub fn parse_action(action_type: &str, arg: &str) -> Action {
    match action_type {
        "reboot" => Action::Reboot,
        "restart-unit" => Action::RestartUnit(arg.to_string()),
        "command" => Action::Command(arg.to_string()),
        other => {
            log::warn!("Unknown action type '{}', falling back to command with given arg", other);
            Action::Command(arg.to_string())
        }
    }
}

fn do_reboot(dry_run: bool) -> Result<()> {
    if dry_run {
        info!("[dry-run] would request system reboot via logind");
        return Ok(());
    }

    // Talk to system bus and call org.freedesktop.login1.Manager.Reboot(true)
    let conn = Connection::system().context("failed to connect to system bus")?;
    let proxy = zbus::blocking::Proxy::new(&conn, "org.freedesktop.login1", "/org/freedesktop/login1", "org.freedesktop.login1.Manager")?;
    proxy.call_method("Reboot", &(true)).context("Reboot call failed")?;
    info!("Reboot requested via logind");
    Ok(())
}

fn do_restart_unit(unit: &str, dry_run: bool) -> Result<()> {
    if dry_run {
        info!("[dry-run] would request restart of unit '{}' via systemd D-Bus", unit);
        return Ok(());
    }

    let conn = Connection::system().context("failed to connect to system bus")?;
    let proxy = zbus::blocking::Proxy::new(&conn, "org.freedesktop.systemd1", "/org/freedesktop/systemd1", "org.freedesktop.systemd1.Manager")?;
    // Mode can be "replace"
    proxy.call_method("RestartUnit", &(unit, "replace"))?;
    info!("Requested restart of unit '{}'", unit);
    Ok(())
}

/// Execute the configured action. Any errors are logged; this function does not return them.
pub fn run_action(action: &Action, dry_run: bool) {
    match action {
        Action::Reboot => {
            if let Err(e) = do_reboot(dry_run) {
                error!("Reboot action failed: {}", e);
            }
        }
        Action::RestartUnit(unit) => {
            if let Err(e) = do_restart_unit(unit, dry_run) {
                error!("Restart unit action failed: {}", e);
            }
        }
        Action::Command(cmd) => {
            if dry_run {
                info!("[dry-run] would run command: {}", cmd);
                return;
            }

            // Fallback to executing the external command. This is the only place we use
            // external processes; prefer D-Bus actions where possible for a pure-Rust workflow.
            match Command::new("sh").arg("-c").arg(cmd).status() {
                Ok(s) => {
                    if s.success() {
                        info!("Command executed successfully");
                    } else {
                        error!("Command exited with status: {}", s);
                    }
                }
                Err(e) => {
                    error!("Failed to execute command: {}", e);
                }
            }
        }
    }
}
