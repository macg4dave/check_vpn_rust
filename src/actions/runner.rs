use super::{Action, PreparedAction};
use anyhow::{bail, Context, Result};
use log::info;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

pub trait ActionRunner {
    fn execute(&self, action: &Action, dry_run: bool) -> Result<()>;
}

/// systemctl uses systemd's D-Bus authorization itself; no parallel D-Bus client is needed.
pub struct RealActionRunner;
impl Default for RealActionRunner {
    fn default() -> Self {
        Self::new()
    }
}
impl RealActionRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn execute_prepared(
        &self,
        prepared: &PreparedAction,
        dry_run: bool,
        timeout: Duration,
        cancel: &AtomicBool,
    ) -> Result<()> {
        if cancel.load(Ordering::Relaxed) {
            bail!("action cancelled");
        }
        if dry_run {
            info!("[dry-run] would execute {:?}", prepared.action);
            return Ok(());
        }
        let mut command = match &prepared.action {
            Action::Log => {
                info!("VPN lost; log-only action");
                return Ok(());
            }
            Action::Command(_) => {
                let parts = prepared.command.as_ref().context("command not prepared")?;
                let mut command = Command::new(&parts[0]);
                command.args(&parts[1..]);
                command
            }
            Action::RestartUnit(unit) => {
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = unit;
                    bail!("restart-unit not supported on this platform");
                }
                #[cfg(target_os = "linux")]
                {
                    let mut command = Command::new("systemctl");
                    command.args(["--no-ask-password", "restart", "--", unit]);
                    command
                }
            }
            Action::Reboot => {
                #[cfg(target_os = "linux")]
                {
                    let mut command = Command::new("systemctl");
                    command.args(["--no-ask-password", "reboot", "--no-block"]);
                    command
                }
                #[cfg(windows)]
                {
                    let mut command = Command::new("shutdown");
                    command.args(["/r", "/t", "0"]);
                    command
                }
                #[cfg(not(any(target_os = "linux", windows)))]
                {
                    let mut command = Command::new("shutdown");
                    command.args(["-r", "now"]);
                    command
                }
            }
        };
        run_process(&mut command, timeout, cancel)
    }
}

/// Poll the owned child so cancellation and timeouts never wait for its normal completion.
pub fn run_process(command: &mut Command, timeout: Duration, cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        bail!("action cancelled");
    }
    struct OwnedChild(std::process::Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut child = OwnedChild(
        command
            .spawn()
            .context("failed to spawn recovery command")?,
    );
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .0
            .try_wait()
            .context("failed to poll recovery command")?
        {
            if status.success() {
                return Ok(());
            }
            bail!("command exited with non-zero status: {status}");
        }
        let cancelled = cancel.load(Ordering::Relaxed);
        if cancelled || started.elapsed() >= timeout {
            if cancelled {
                bail!("action cancelled");
            }
            bail!("action timed out after {} seconds", timeout.as_secs_f64());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
impl ActionRunner for RealActionRunner {
    fn execute(&self, action: &Action, dry_run: bool) -> Result<()> {
        // Dry-run compatibility for library callers: never spawn a process.
        if dry_run {
            info!("[dry-run] would execute {action:?}");
            return Ok(());
        }
        let prepared = PreparedAction::new(action.clone())?;
        self.execute_prepared(
            &prepared,
            false,
            Duration::from_secs(30),
            &AtomicBool::new(false),
        )
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    #[cfg(not(windows))]
    fn command_nonzero_status_returns_error() {
        // Use a portable non-zero command on Unix-like systems.
        let runner = RealActionRunner::new();
        let res = runner.execute(&Action::Command("false".into()), false);
        assert!(
            res.is_err(),
            "expected non-zero command to be surfaced as error"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn command_parses_quoted_arguments() {
        // Ensure quoted args are preserved (no shell is used).
        let runner = RealActionRunner::new();
        let cmd = r#"/bin/echo "hi there""#;
        let res = runner.execute(&Action::Command(cmd.into()), false);
        assert!(res.is_ok(), "expected quoted command to execute");
    }

    #[test]
    #[cfg(not(windows))]
    fn command_with_meta_characters_is_rejected() {
        let runner = RealActionRunner::new();
        let cmd = "echo ok; rm -rf /";
        let res = runner.execute(&Action::Command(cmd.into()), false);
        assert!(
            res.is_err(),
            "commands containing shell metacharacters must be rejected"
        );
    }

    #[test]
    #[cfg(unix)]
    fn command_supports_spaced_program_path() {
        use std::fs::File;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let script_path = dir
            .path()
            .join("script with spaces.sh")
            .to_string_lossy()
            .to_string();

        let mut f = File::create(&script_path).expect("create script");
        writeln!(f, "#!/bin/sh\nexit 0").expect("write script");
        drop(f);
        let mut perms = std::fs::metadata(&script_path)
            .expect("metadata")
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("chmod");

        let runner = RealActionRunner::new();
        let cmd = format!("\"{}\"", script_path);
        let res = runner.execute(&Action::Command(cmd), false);
        assert!(res.is_ok(), "spaced program paths should be supported");
    }
}
