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
pub struct RealActionRunner {
    /// Whether we successfully detected a usable system D-Bus at construction time.
    /// When false, runner avoids D-Bus calls and will fall back to shell or no-op.
    dbus_available: bool,
}

impl RealActionRunner {
    pub fn new() -> Self {
        // On non-Linux platforms we do not expect a system D-Bus to be present.
        if !cfg!(target_os = "linux") {
            info!(
                "Non-Linux platform detected — D-Bus considered unavailable; using shell/no-op fallbacks"
            );
            return RealActionRunner {
                dbus_available: false,
            };
        }

        // Best-effort detection on Linux.
        match Connection::system() {
            Ok(_) => RealActionRunner {
                dbus_available: true,
            },
            Err(e) => {
                info!(
                    "Failed to connect to system D-Bus — will fall back to shell/no-op actions: {:?}",
                    e
                );
                RealActionRunner {
                    dbus_available: false,
                }
            }
        }
    }

    /// Construct a test-friendly runner that treats D-Bus availability as
    /// specified by the caller. This is only compiled for tests.
    #[cfg(test)]
    pub fn new_with_dbus_available(dbus_available: bool) -> Self {
        RealActionRunner { dbus_available }
    }
    // A Default impl is convenient for callers/tests that want the zero-config
    // runner without calling `new` explicitly. Clippy suggests adding this.

    fn do_reboot(&self, dry_run: bool) -> Result<()> {
        if dry_run {
            if self.dbus_available {
                info!("[dry-run] would request system reboot via logind");
            } else {
                info!("[dry-run] would request system reboot via fallback (no D-Bus)");
            }
            return Ok(());
        }

        if self.dbus_available {
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
        } else {
            info!("D-Bus unavailable; running explicit fallback for reboot");
            // Try systemctl first, then fallback to the reboot binary.
            match Command::new("systemctl")
                .arg("reboot")
                .arg("--no-block")
                .status()
            {
                        Ok(s) => {
                    if s.success() {
                        info!("Fallback reboot command executed successfully");
                    } else {
                        error!("Fallback reboot command returned non-zero: {:?}", s);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to execute fallback reboot command: {:?}", e);
                    // try fallback reboot binary if systemctl is missing or failed to spawn
                    match Command::new("reboot").status() {
                        Ok(r) => {
                            if r.success() {
                                info!("Fallback reboot binary executed successfully");
                                Ok(())
                            } else {
                                error!("Fallback reboot binary returned non-zero: {:?}", r);
                                Err(anyhow::anyhow!("reboot returned non-zero status"))
                                    .context("failed to run any reboot fallback")
                            }
                        }
                        Err(e2) => {
                            error!("Failed to execute fallback reboot binary: {:?}", e2);
                            Err(e2).context("failed to spawn reboot fallback command")
                        }
                    }
                }
            }
        }
    }

    fn do_restart_unit(&self, unit: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            if self.dbus_available {
                info!(
                    "[dry-run] would request restart of unit '{}' via systemd D-Bus",
                    unit
                );
            } else {
                info!(
                    "[dry-run] would request restart of unit '{}' via fallback (no D-Bus)",
                    unit
                );
            }
            return Ok(());
        }

        if self.dbus_available {
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
        } else {
            info!("D-Bus unavailable; running explicit fallback for restart");
            // Basic validation: unit name must not contain whitespace or obvious
            // shell meta-characters.
            if !crate::config::validation::is_valid_unit_name(unit) {
                error!("Refusing to restart unit with unsafe unit name: '{}'", unit);
                return Err(anyhow::anyhow!("unsafe unit name").context("invalid unit name"));
            }
            match Command::new("systemctl").arg("restart").arg(unit).status() {
                Ok(s) => {
                    if s.success() {
                        info!("Fallback restart command executed successfully");
                    } else {
                        error!("Fallback restart command returned non-zero: {:?}", s);
                    }
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to execute fallback restart command: {:?}", e);
                    Err(e).context("failed to spawn restart fallback command")
                }
            }
        }
    }

    fn do_command(&self, cmd: &str, dry_run: bool) -> Result<()> {
        if dry_run {
            info!("[dry-run] would run command: {}", cmd);
            return Ok(());
        }

        // Fallback to executing the external command. We will not use a shell
        // trampoline — instead split the command on whitespace into an executable
        // and its arguments and execute directly. Also perform a lightweight
        // validation to reject strings that contain obvious shell meta-characters.
        if !crate::config::validation::is_safe_command_string(cmd) {
            error!("Command contains forbidden shell meta-characters; refusing to run");
            return Err(anyhow::anyhow!("command contains forbidden shell meta-characters").context("unsafe command"));
        }

        let mut parts = cmd.split_whitespace();
        let prog = match parts.next() {
            Some(p) => p,
            None => {
                error!("Empty command passed to runner");
                return Err(anyhow::anyhow!("empty command").context("no executable found"));
            }
        };

        match Command::new(prog).args(parts).status() {
            Ok(s) => {
                if s.success() {
                    info!("Command executed successfully");
                    Ok(())
                } else {
                    error!("Command exited with status: {:?}", s);
                    Ok(()) // do not treat non-zero as a hard error for now
                }
            }
            Err(e) => {
                error!("Failed to execute command: {:?}", e);
                Err(e).context("failed to spawn command")
            }
        }
    }
}

impl Default for RealActionRunner {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn dry_run_behaviour_with_dbus_unavailable() {
        // Simulate an environment where D-Bus is not available
        let runner = RealActionRunner::new_with_dbus_available(false);

        // All actions in dry-run must return Ok and not execute side-effects
        assert!(runner.execute(&Action::Reboot, true).is_ok());
        assert!(runner
            .execute(&Action::RestartUnit("fake-unit".into()), true)
            .is_ok());
        assert!(runner
            .execute(&Action::Command("echo hi".into()), true)
            .is_ok());
    }

    #[test]
    fn dry_run_behaviour_with_dbus_available() {
        // Simulate an environment where D-Bus is available — dry-run should still be safe
        let runner = RealActionRunner::new_with_dbus_available(true);

        assert!(runner.execute(&Action::Reboot, true).is_ok());
        assert!(runner
            .execute(&Action::RestartUnit("fake-unit".into()), true)
            .is_ok());
        assert!(runner
            .execute(&Action::Command("echo hi".into()), true)
            .is_ok());
    }
}
