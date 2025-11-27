use clap::Parser;
use std::fs;

#[test]
fn generate_completions_writes_file() {
    // Create a temp file path and ask the generator to write to it.
    let path = std::env::temp_dir().join("check_vpn_comp_test.out");
    let path_str = path.to_str().unwrap().to_string();

    // Generate bash completions into the temp file
    check_vpn::completions::generate_completions("bash", Some(path_str.as_str()))
        .expect("generate comps");

    assert!(
        std::path::Path::new(&path_str).exists(),
        "completions file should be written"
    );
    let contents = fs::read_to_string(&path_str).expect("read comps");
    // A completion file for bash should contain the program name
    assert!(contents.contains("check_vpn"));
    let _ = fs::remove_file(path);
}

#[test]
fn generate_completions_unsupported_shell_errors() {
    // Unsupported shells must return an error
    let res = check_vpn::completions::generate_completions("fish", None);
    assert!(res.is_err());
}

#[test]
fn cli_parsing_dry_run_flag() {
    // Ensure the CLI parsing honors --dry-run and run_once semantics via Args::parse_from
    let argv = vec!["check_vpn", "--dry-run", "--run-once"];
    let args = check_vpn::cli::Args::parse_from(argv);
    assert!(args.dry_run);
    assert!(args.run_once);
}
