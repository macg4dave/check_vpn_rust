use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn binary_exit_code_on_malformed_config() {
    // Create a malformed XML file so config loader returns an error.
    let mut f = NamedTempFile::new().expect("temp file");
    writeln!(f, "Not an XML").unwrap();
    let path = f.path().to_str().unwrap().to_string();

    // Run the built binary with the malformed config path.
    let bin = std::env::var("CARGO_BIN_EXE_check_vpn").unwrap_or_else(|_| {
        if cfg!(windows) {
            "target\\debug\\check_vpn.exe".to_string()
        } else {
            "target/debug/check_vpn".to_string()
        }
    });
    let mut cmd = Command::new(bin);
    cmd.arg("--run-once");
    cmd.env("CHECK_VPN_CONFIG", &path);
    let status = cmd.status().expect("failed to execute binary");
    // Expect the exit code to be the invalid-config code (2)
    assert_eq!(status.code().unwrap_or(-1), 2);
}

#[test]
fn binary_exit_code_on_connectivity_failure() {
    let bin = std::env::var("CARGO_BIN_EXE_check_vpn").unwrap_or_else(|_| {
        if cfg!(windows) {
            "target\\debug\\check_vpn.exe".to_string()
        } else {
            "target/debug/check_vpn".to_string()
        }
    });

    let mut cmd = Command::new(bin);
    // Force an unavailable provider; TCP probes no longer gate successful identity queries.
    cmd.args([
        "--disable-ip-api",
        "--disable-ifconfig-co",
        "--provider-url",
        "http://127.0.0.1:1/json",
    ]);
    // use run-once + exit-on-error and an unlikely open port on localhost to force failure
    cmd.arg("--run-once")
        .arg("--exit-on-error")
        .arg("--connectivity-endpoint")
        .arg("127.0.0.1")
        .arg("--connectivity-ports")
        .arg("65535");

    let status = cmd.status().expect("failed to execute binary");
    assert_eq!(status.code().unwrap_or(-1), 1);
}
