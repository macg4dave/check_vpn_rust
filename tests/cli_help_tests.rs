use clap::CommandFactory;
use check_vpn::cli::Args;

#[test]
fn help_mentions_config_flag_and_placeholder() {
    // Render the CLI help into a buffer and check for the presence of
    // the `--config` flag and a value placeholder. Different platforms or
    // clap versions may use `<FILE>` or `<PATH>`; accept either.
    let mut buf = Vec::new();
    Args::command().write_long_help(&mut buf).expect("write help");
    let s = String::from_utf8(buf).expect("utf8");

    assert!(s.contains("--config"), "help should mention --config flag\n\n{}", s);
    // Accept either <FILE> or <PATH> as the placeholder used by clap
    assert!(s.contains("<FILE>") || s.contains("<PATH>"), "help should show a value placeholder (<FILE> or <PATH>)\n\n{}", s);
}
