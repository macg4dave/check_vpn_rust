use std::fs;
mod support;

#[test]
fn init_wizard_writes_file_with_defaults() {
    // Use no-fetch to avoid real network calls in tests
    let (_td, path) = support::temp_dir_with_child("init_wizard", "check_vpn_gen_test.xml");
    let path_str = path.to_str().unwrap().to_string();

    // Force non-interactive mode to ensure the test never blocks on user input
    check_vpn::init_wizard::set_no_prompt_for_tests(true);

    check_vpn::init_wizard::run_init(Some(path_str.as_str()), true).expect("init wizard");

    assert!(
        std::path::Path::new(&path_str).exists(),
        "file should be written"
    );
    let contents = fs::read_to_string(&path_str).expect("read file");
    assert!(contents.contains("<config>"));
    assert!(contents.contains("<interval>"));
    // cleanup happens when `_td` is dropped
}

#[test]
fn run_with_timeout_times_out_and_returns_err() {
    // Use a long-running closure and a very short timeout.
    let res = check_vpn::init_wizard::run_with_timeout(
        |cancel| {
            while !cancel.load(std::sync::atomic::Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            1u32
        },
        std::time::Duration::from_millis(10),
    );
    assert!(res.is_err(), "expected timeout to return Err");
}
