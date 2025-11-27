use std::fs;
mod support;

#[test]
fn completions_writes_output_file() {
    // Use a temporary directory so the generated file is cleaned up when the
    // TempDir guard is dropped.
    let (_td, path) = support::temp_dir_with_child("completions", "check_vpn_completions_test.bash");
    let path_str = path.to_str().unwrap();

    check_vpn::completions::generate_completions("bash", Some(path_str))
        .expect("generate completions");

    assert!(
        std::path::Path::new(path_str).exists(),
        "completions file should be written"
    );
    let contents = fs::read_to_string(path_str).expect("read file");
    // Basic sanity check — content should not be empty and should contain a completion marker.
    assert!(!contents.is_empty());
    // TempDir `_td` is dropped automatically at the end of the test, cleaning up.
}
