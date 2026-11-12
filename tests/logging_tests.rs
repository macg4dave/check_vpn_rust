use std::sync::atomic::{AtomicBool, Ordering};

static PANIC_OBSERVED: AtomicBool = AtomicBool::new(false);

// Run global logger and panic-hook checks in a child process so they cannot
// affect the test harness or other tests running concurrently.
#[test]
fn logging_process_fixture() {
    if std::env::var_os("CHECK_VPN_LOGGING_FIXTURE").is_none() {
        return;
    }
    let before = std::env::var_os("RUST_LOG");
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {
        PANIC_OBSERVED.store(true, Ordering::Relaxed);
    }));
    check_vpn::logging::init_with_verbosity(1);
    check_vpn::logging::init_with_verbosity(2);
    assert_eq!(std::env::var_os("RUST_LOG"), before);
    assert!(std::panic::catch_unwind(|| panic!("hook probe")).is_err());
    assert!(PANIC_OBSERVED.load(Ordering::Relaxed));
    std::panic::set_hook(previous_hook);
    log::debug!("logging regression debug probe");
}

#[test]
fn logging_preserves_environment_hook_and_filter() {
    for configured in [None, Some("error")] {
        let mut child =
            std::process::Command::new(std::env::current_exe().expect("test executable"));
        child
            .args(["--exact", "logging_process_fixture", "--nocapture"])
            .env("CHECK_VPN_LOGGING_FIXTURE", "1");
        match configured {
            Some(value) => {
                child.env("RUST_LOG", value);
            }
            None => {
                child.env_remove("RUST_LOG");
            }
        }
        let output = child.output().expect("run isolated logging check");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "logging child failed: {stderr}");
        assert_eq!(
            stderr.contains("logging regression debug probe"),
            configured.is_none(),
            "{stderr}"
        );
    }
}
