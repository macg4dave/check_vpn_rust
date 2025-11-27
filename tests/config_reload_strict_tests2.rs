use std::fs;
// write handled by temporary helper
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

mod support;

fn spawn_binary_with_config(args: &[&str]) -> std::process::Child {
    let bin = std::env::var("CARGO_BIN_EXE_check_vpn").unwrap_or_else(|_| {
        let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.push("target");
        p.push("debug");
        p.push("check_vpn");
        p.to_str().unwrap().to_string()
    });
    let mut cmd = Command::new(bin);
    for a in args.iter() {
        cmd.arg(a);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.spawn().expect("spawn check_vpn")
}

fn spawn_output_collector(child: &mut std::process::Child) -> Receiver<String> {
    let (tx, rx): (Sender<String>, Receiver<String>) = channel();
    if let Some(stdout) = child.stdout.take() {
        let tx2 = tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = tx2.send(line.clone());
                    }
                    Err(_) => break,
                }
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let tx2 = tx.clone();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) => break,
                    Ok(_) => {
                        let _ = tx2.send(line.clone());
                    }
                    Err(_) => break,
                }
            }
        });
    }
    rx
}

fn wait_for_collector_contains(rx: &Receiver<String>, substr: &str, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(line) = rx.recv_timeout(Duration::from_millis(200)) {
            if line.contains(substr) {
                return true;
            }
        }
    }
    false
}

#[test]
fn reload_continues_without_strict_flag() {
    let (_listener, port) = support::tcp_listener_and_port();

    let xml = r#"<config><interval>1</interval><isp_to_check>TestISP</isp_to_check></config>"#;
    let (_td, p) = support::temp_dir_with_file("reload", "check_vpn_it_reload.xml", xml.as_bytes());
    let path_str = p.to_str().unwrap().to_string();

    let mut child = spawn_binary_with_config(&[
        "--config",
        &path_str,
        "--interval",
        "1",
        "--dry-run",
        "--connectivity-endpoint",
        "127.0.0.1",
        "--connectivity-ports",
        &format!("{}", port),
        "--disable-ip-api",
        "--disable-ifconfig-co",
    ]);

    let rx = spawn_output_collector(&mut child);
    assert!(
        wait_for_collector_contains(&rx, "Starting check_vpn run()", Duration::from_secs(10)),
        "timed out waiting for child to start"
    );

    // Corrupt the config file; this should NOT cause the binary to exit when not strict.
    fs::write(&path_str, "<bad>").expect("corrupt file");

    thread::sleep(Duration::from_secs(3));
    if let Ok(Some(status)) = child.try_wait() {
        let out = child
            .wait_with_output()
            .expect("failed to get child output");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        panic!(
            "Binary unexpectedly exited without --strict-config (status={:?})\nstdout:\n{}\nstderr:\n{}",
            status, stdout, stderr
        );
    }

    let _ = child.kill();
    // TempDir `_td` drops and cleans up
}

#[test]
fn reload_exits_with_strict_flag() {
    let (_listener, port) = support::tcp_listener_and_port();

    let xml = r#"<config><interval>1</interval><isp_to_check>TestISP</isp_to_check></config>"#;
    let (_td, p) = support::temp_dir_with_file("reload_strict", "check_vpn_it_reload_strict.xml", xml.as_bytes());
    let path_str = p.to_str().unwrap().to_string();

    let mut child = spawn_binary_with_config(&[
        "--config",
        &path_str,
        "--interval",
        "1",
        "--dry-run",
        "--connectivity-endpoint",
        "127.0.0.1",
        "--connectivity-ports",
        &format!("{}", port),
        "--strict-config",
        "--disable-ip-api",
        "--disable-ifconfig-co",
    ]);

    let rx = spawn_output_collector(&mut child);
    assert!(
        wait_for_collector_contains(&rx, "Starting check_vpn run()", Duration::from_secs(10)),
        "timed out waiting for child to start"
    );
    thread::sleep(Duration::from_secs(2));

    fs::write(&path_str, "<bad>").expect("corrupt file");

    // Wait for the strict-config message and for the process to exit
    if !wait_for_collector_contains(&rx, "--strict-config is set", Duration::from_secs(20)) {
        // collect remaining lines for diagnostics
        let mut lines: Vec<String> = Vec::new();
        while let Ok(line) = rx.recv_timeout(Duration::from_millis(100)) {
            lines.push(line);
        }
        panic!(
            "Expected reload error message about strict-config but did not see it in logs; last lines:\n{}",
            lines.join("\n")
        );
    }

    let start = Instant::now();
    let mut exited = None;
    while start.elapsed() < Duration::from_secs(8) {
        if let Ok(Some(status)) = child.try_wait() {
            let out = child
                .wait_with_output()
                .expect("failed to get child output");
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            exited = Some((status, stdout.to_string(), stderr.to_string()));
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }

    let (status, stdout, stderr) =
        exited.expect("process did not exit after strict reload failure");
    assert!(
        !status.success(),
        "Process exited zero; expected non-zero due to strict-config. stdout:\n{}\nstderr:\n{}",
        stdout,
        stderr
    );
    // TempDir `_td` drops and cleans up
}
