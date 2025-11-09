use anyhow::{Context, Result};
use log::{error, info};
use std::process::Command;
use zbus::blocking::Connection;

use super::Action;

/// Trait allowing injection of action execution implementations for testing.
///
/// Implementors should perform the requested action and return an error on failure.
pub trait ActionRunner {
    fn execute(&self, action: &Action, dry_run: bool) -> Result<()>;
}

/// Real production implementation that talks to systemd/logind over D-Bus or runs shell commands.
pub struct RealActionRunner {}

impl RealActionRunner {
    pub fn new() -> Self {
        RealActionRunner {}
    }

    fn do_reboot(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            info!("[dry-run] would request system reboot via logind");
            return Ok(());
        }

        // Talk to system bus and call org.freedesktop.login1.Manager.Reboot(true)
        let conn = Connection::system().context("failed to connect to system bus")?;
        let proxy = zbus::blocking::Proxy::new(
            &conn,
            "org.freedesktop.login1",
            "/org/freedesktop/login1",
            "org.freedesktop.login1.Manager",
        )?;
        proxy
            .call_method("Reboot", &(true))
            .context("Reboot call failed")?;
        info!("Reboot requested via logind");
        Ok(())
    }

    fn do_restart_unit(&self, unit: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            info!("[dry-run] would request restart of unit '{}' via systemd D-Bus", unit);
            return Ok(());
        }

        let conn = Connection::system().context("failed to connect to system bus")?;
        let proxy = zbus::blocking::Proxy::new(
            &conn,
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.systemd1.Manager",
        )?;
        // Mode can be "replace"
        proxy.call_method("RestartUnit", &(unit, "replace"))?;
        info!("Requested restart of unit '{}'", unit);
        Ok(())
    }

    fn do_command(&self, cmd: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            info!("[dry-run] would run command: {}", cmd);
            return Ok(());
        }

        // Fallback to executing the external command. This is the only place we use
        // external processes; prefer D-Bus actions where possible for a pure-Rust workflow.
        match Command::new("sh").arg("-c").arg(cmd).status() {
            Ok(s) => {
                if s.success() {
                    info!("Command executed successfully");
                    Ok(())
                } else {
                    error!("Command exited with status: {}", s);
                    Ok(()) // do not treat non-zero as a hard error for now
                }
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
                Err(e).context("failed to spawn command")
            }
        }
    }
}

impl ActionRunner for RealActionRunner {
    fn execute(&self, action: &Action, dry_run: bool) -> Result<()> {
        match action {
            Action::Reboot => self.do_reboot(dry_run),
            Action::RestartUnit(unit) => self.do_restart_unit(unit, dry_run),
            Action::Command(cmd) => self.do_command(cmd, dry_run),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PanicRunner {}
    impl ActionRunner for PanicRunner {
        fn execute(&self, _action: &Action, _dry_run: bool) -> Result<()> {
            panic!("should not be called")
        }
    }

    #[test]
    fn runner_trait_object_dispatch_works() {
        let r: &dyn ActionRunner = &RealActionRunner::new();
        // Using dry_run true to prevent side-effects on CI/dev machines
        assert!(r.execute(&Action::Command("echo hi".into()), true).is_ok());

        let res = std::panic::catch_unwind(|| {
            // construct the panic runner inside the closure so we don't capture a
            // `&dyn ActionRunner` across the unwind boundary (requires RefUnwindSafe).
            let p = PanicRunner {};
            p.execute(&Action::Command("echo".into()), true)
        });
        assert!(res.is_err());
    }
}
