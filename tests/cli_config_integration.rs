mod support;
use std::process::Command;

#[test]
fn binary_accepts_config_flag() {
    let xml = r#"<config><interval>1</interval><isp_to_check>TestISP</isp_to_check></config>"#;
    let (_dir, path) = support::temp_dir_with_file("cli_it", "config.xml", xml.as_bytes());
    let server = support::start_http_server(r#"{"isp":"Other ISP"}"#, 200);
    let output = Command::new(env!("CARGO_BIN_EXE_check_vpn"))
        .arg("--config")
        .arg(&path)
        .args([
            "--run-once",
            "--dry-run",
            "--disable-ip-api",
            "--disable-ifconfig-co",
            "--provider-url",
            &server.url(),
        ])
        .output()
        .expect("spawn check_vpn");

    assert!(
        output.status.success(),
        "binary failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
