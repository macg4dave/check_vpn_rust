use std::fs;

#[test]
fn dockerignore_exists_and_excludes_target() {
    let s = fs::read_to_string(".dockerignore").expect(".dockerignore must exist at repo root for safe docker builds");
    // Make sure we don't accidentally send build artifacts like target/ in the
    // context — this keeps Docker build contexts small and prevents BuildKit
    // failures for very large repos.
    assert!(s.contains("target/"), ".dockerignore should exclude 'target/'");
}
