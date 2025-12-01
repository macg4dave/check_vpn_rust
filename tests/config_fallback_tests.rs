mod support;

#[test]
fn load_from_path_errors_on_malformed_xml() {
    // Create a malformed XML file
    let (_td, path) =
        support::temp_dir_with_file("malformed", "check_vpn_malformed.xml", b"NOT XML HERE");

    let res = check_vpn::config::Config::load_from_path(&path);
    assert!(res.is_err(), "malformed xml should return an error");

    // TempDir `_td` drops and cleans up
}

#[test]
fn load_returns_default_when_no_file_present() {
    // Create a fresh temp directory and ensure no check_vpn.xml exists
    let td = tempfile::tempdir().expect("tempdir");
    let prev = std::env::current_dir().expect("curdir");
    std::env::set_current_dir(td.path()).expect("chdir");

    let cfg = check_vpn::config::Config::load().expect("load should succeed and return default");
    // default config should contain some sensible defaults
    assert!(cfg.interval.is_some());

    // restore working directory
    std::env::set_current_dir(prev).expect("restore cwd");
}
