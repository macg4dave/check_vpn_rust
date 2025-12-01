use std::path::Path;

#[test]
fn packaging_scripts_and_dockerfiles_exist() {
    // Ensure the packaging orchestrator and Dockerfiles are present in the repo.
    let script = Path::new("./scripts/build-all-packages.sh");
    assert!(script.exists(), "build-all-packages.sh must exist");

    let df_deb = Path::new("./contrib/Dockerfile.debian-packaging");
    let df_fed = Path::new("./contrib/Dockerfile.fedora-packaging");
    assert!(
        df_deb.exists(),
        "contrib/Dockerfile.debian-packaging must exist"
    );
    assert!(
        df_fed.exists(),
        "contrib/Dockerfile.fedora-packaging must exist"
    );

    // Ensure the script creates a unix layout for other Linuxes (binary + systemd etc)
    let script_contents = std::fs::read_to_string(script).expect("read script");
    assert!(
        script_contents.contains("/unix/") || script_contents.contains("unix/"),
        "script should create release/unix/<arch> output"
    );
}
