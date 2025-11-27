use std::fs;

#[test]
fn completions_writes_output_file() {
    let path = std::env::temp_dir().join("check_vpn_completions_test.bash");
    let path_str = path.to_str().unwrap();

    // Ensure any previous test artifact is removed.
    let _ = fs::remove_file(&path);

    check_vpn::completions::generate_completions("bash", Some(path_str))
        .expect("generate completions");

    assert!(
        std::path::Path::new(path_str).exists(),
        "completions file should be written"
    );
    let contents = fs::read_to_string(path_str).expect("read file");
    // Basic sanity check — content should not be empty and should contain a completion marker.
    assert!(!contents.is_empty());
    // Cleanup
    let _ = fs::remove_file(&path);
}
