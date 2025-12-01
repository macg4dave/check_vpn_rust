#[cfg(unix)]
use std::fs;
mod support;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn make_temp_file(name: &str, contents: &[u8], make_exec: bool) -> tempfile::NamedTempFile {
    // create a NamedTempFile via the test helper. Keep the file guard alive
    // by returning it — the file will be removed when the guard is dropped.
    let f = support::create_temp_file_with_contents(name, contents);
    if make_exec {
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(f.path()).expect("meta").permissions();
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
            fs::set_permissions(f.path(), perms).expect("set perms");
        }
    }
    f
}

#[cfg(unix)]
fn run_installer(args: &[&str]) -> (bool, String, String) {
    let mut cmd = Command::new("bash");
    cmd.arg("scripts/install.sh");
    for a in args {
        cmd.arg(a);
    }
    let output = cmd.output().expect("run installer");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

#[cfg(unix)]
#[test]
fn test_dry_run_user_installs_binary_and_config() {
    // create temp binary and config
    let bin = make_temp_file("bin", b"#! /bin/sh\n", true);
    let config = make_temp_file("config", b"<config/>\n", false);

    let bin_path = bin.path().to_string_lossy().into_owned();
    let config_path = config.path().to_string_lossy().into_owned();

    let (ok, out, err) = run_installer(&[
        "--binary",
        &bin_path,
        "--config",
        &config_path,
        "--mode",
        "user",
        "--yes",
        "--dry-run",
    ]);
    assert!(ok, "installer failed: stderr={} stdout={}", err, out);

    // should show dry-run copy for binary and config
    assert!(
        out.contains(&bin_path),
        "stdout should mention binary path: {}",
        out
    );
    assert!(
        out.contains(&config_path),
        "stdout should mention config path: {}",
        out
    );
    assert!(
        out.contains("DRY-RUN"),
        "stdout should include DRY-RUN hints: {}",
        out
    );
    assert!(
        out.contains("Installed binary to"),
        "should still log installation target"
    );
    assert!(out.contains("Installed config to") || out.contains("No config found"));
}

#[cfg(unix)]
#[test]
fn test_dry_run_service_flag_on_linux_if_applicable() {
    // create temp binary and service
    let bin = make_temp_file("bin2", b"#! /bin/sh\n", true);
    let service = make_temp_file("service", b"[Unit]\nDescription=check_vpn\n", false);

    let bin_path = bin.path().to_string_lossy().into_owned();
    let service_path = service.path().to_string_lossy().into_owned();

    // run in user mode to avoid needing sudo
    let (ok, out, err) = run_installer(&[
        "--binary",
        &bin_path,
        "--service",
        &service_path,
        "--mode",
        "user",
        "--yes",
        "--dry-run",
    ]);
    assert!(ok, "installer failed: stderr={} stdout={}", err, out);

    // Always expect the binary to be mentioned
    assert!(out.contains(&bin_path));

    // On Linux with systemctl present, the script will copy the provided service file for user installs
    if cfg!(target_os = "linux") {
        // check if systemctl is available in PATH
        let has_systemctl = Command::new("sh")
            .arg("-c")
            .arg("command -v systemctl >/dev/null 2>&1 && echo yes || echo no")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().eq("yes"))
            .unwrap_or(false);
        if has_systemctl {
            assert!(
                out.contains(&service_path),
                "service file should be referenced in dry-run output"
            );
        }
    }
}

#[cfg(unix)]
#[test]
fn test_system_install_requires_root() {
    // If tests are running as root, skip this check because it expects non-root failure.
    let uid_out = Command::new("sh")
        .arg("-c")
        .arg("id -u")
        .output()
        .expect("failed to get uid");
    let uid = String::from_utf8_lossy(&uid_out.stdout).trim().to_string();
    if uid == "0" {
        eprintln!("Skipping test_system_install_requires_root because running as root");
        return;
    }

    let bin = make_temp_file("bin_sys", b"#! /bin/sh\n", true);
    let bin_path = bin.path().to_string_lossy().into_owned();

    // Run in system mode as non-root; should fail with an error requesting root.
    let (ok, out, err) = run_installer(&[
        "--binary",
        &bin_path,
        "--mode",
        "system",
        "--yes",
        "--dry-run",
    ]);
    assert!(!ok, "Expected installer to fail when system install is attempted without root. stdout={} stderr={}", out, err);
    let combined = format!("{}{}", out, err);
    assert!(
        combined.contains("requires root") || combined.contains("System install requires root"),
        "Expected message about requiring root; got: {}",
        combined
    );
}
