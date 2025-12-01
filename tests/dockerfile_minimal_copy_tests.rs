use std::fs;

fn check_minimal_copy(path: &str) {
    let s = fs::read_to_string(path).unwrap_or_else(|_| panic!("unable to read {}", path));

    // Ensure we first copy Cargo.toml and Cargo.lock
    assert!(
        s.contains("COPY Cargo.toml Cargo.lock"),
        "{} must contain 'COPY Cargo.toml Cargo.lock'",
        path
    );

    // Ensure a small 'seed' src/dummy build step exists
    assert!(
        s.contains("mkdir -p src") || s.contains("mkdir src"),
        "{} must seed a dummy src (mkdir -p src)",
        path
    );

    // Find the first non-comment 'COPY Cargo.toml' instruction and the first
    // non-comment selective 'COPY src' instruction and verify ordering.
    // Avoid accidental matches inside comment text.
    let mut manifest_line: Option<usize> = None;
    let mut src_copy_line: Option<usize> = None;
    let mut offset = 0usize;
    for line in s.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            if manifest_line.is_none() && trimmed.starts_with("COPY Cargo.toml") {
                manifest_line = Some(offset);
            }
            if src_copy_line.is_none()
                && (trimmed.starts_with("COPY src") || trimmed.starts_with("COPY ./src"))
            {
                src_copy_line = Some(offset);
            }
        }
        // add length of line + newline for offset tracking
        offset += line.len() + 1;
    }

    let idx_manifest = manifest_line.expect("manifest COPY instruction not found");
    let idx_src = src_copy_line.expect("selective 'COPY src' instruction not found");
    assert!(
        idx_manifest < idx_src,
        "{} should COPY Cargo.toml before the selective src COPY",
        path
    );

    // Ensure we copy tests/support rather than the entire tests/ tree
    assert!(
        s.contains("COPY tests/support"),
        "{} should copy tests/support instead of the whole tests/ tree",
        path
    );

    // Ensure the Dockerfile does NOT use `COPY . .` which would send the
    // whole repository into the build context. We want selective copies.
    if s.lines().any(|l| l.trim_start().starts_with("COPY . .")) {
        panic!(
            "{} must not use 'COPY . .' — use selective COPYs instead",
            path
        );
    }
}

#[test]
fn dockerfiles_use_minimal_copy() {
    let files = [
        "contrib/Dockerfile",
        "contrib/Dockerfile.debian",
        "contrib/Dockerfile.fedora",
    ];

    for f in files.iter() {
        check_minimal_copy(f);
    }
}
