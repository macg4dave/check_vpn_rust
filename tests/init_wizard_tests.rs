use std::fs;

#[test]
fn init_wizard_writes_file_with_defaults() {
    // Use no-fetch to avoid real network calls in tests
    let path = std::env::temp_dir().join("check_vpn_gen_test.xml");
    let path_str = path.to_str().unwrap().to_string();

    // Force non-interactive mode to ensure the test never blocks on user input
    check_vpn::init_wizard::set_no_prompt_for_tests(true);

    check_vpn::init_wizard::run_init(Some(path_str.clone()), true).expect("init wizard");

    assert!(std::path::Path::new(&path_str).exists(), "file should be written");
    let contents = fs::read_to_string(&path_str).expect("read file");
    assert!(contents.contains("<config>"));
    assert!(contents.contains("<interval>"));
    let _ = fs::remove_file(path);
}
