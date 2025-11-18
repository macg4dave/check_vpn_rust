use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

// This is a simple integration test that spawns the built binary with
// `--config <path>` pointing to a temporary XML file. We expect the binary
// to start and exit quickly in `--run-once` mode; use `--dry-run` to avoid
// side effects.
#[test]
fn binary_accepts_config_flag() {
    // create a small config xml file
    let xml = r#"<config><interval>1</interval><isp_to_check>TestISP</isp_to_check></config>"#;
    let mut path = std::env::temp_dir();
    path.push(format!("check_vpn_it_{}.xml", std::time::Instant::now().elapsed().as_nanos()));
    let path_str = path.to_str().unwrap().to_string();
    let mut f = fs::File::create(&path_str).expect("create temp xml");
    f.write_all(xml.as_bytes()).expect("write xml");

    // Build path to the test binary in target/debug
    // We assume the binary is already built by the test harness invocation; if not,
    // spawning `cargo run` would complicate running under `cargo test`. Instead,
    // use the env var CARGO_BIN_EXE_check_vpn which cargo sets for integration tests
    // when the binary is built as part of the same workspace. If absent, fallback
    // to target/debug/check_vpn.
    let bin = std::env::var("CARGO_BIN_EXE_check_vpn").unwrap_or_else(|_| {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("target");
        p.push("debug");
        p.push("check_vpn");
        p.to_str().unwrap().to_string()
    });

    // Run binary with --config and --run-once --dry-run so it exits quickly
    let mut cmd = Command::new(bin);
    cmd.arg("--config").arg(&path_str).arg("--run-once").arg("--dry-run");
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::piped());

    let output = cmd.output().expect("spawn check_vpn");

    // cleanup temp file
    let _ = fs::remove_file(&path_str);

    // Binary should exit with code 0 when run-once + dry-run; ensure it started
    // and didn't crash with a missing flag error.
    assert!(output.status.success(), "binary exited non-zero (stderr may contain details)");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // verify the program printed something helpful (e.g., logs). We only check it ran.
    let _ = stdout; // no-op; presence of output is not required — success status is sufficient
}
