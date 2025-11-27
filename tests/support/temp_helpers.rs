use std::io::Write;
use std::path::PathBuf;

/// Create a NamedTempFile containing `contents` and return the NamedTempFile.
/// The returned file is removed when the NamedTempFile is dropped by the caller.
pub fn create_temp_file_with_contents(prefix: &str, contents: &[u8]) -> tempfile::NamedTempFile {
    let mut tf = tempfile::Builder::new()
        .prefix(prefix)
        .tempfile()
        .expect("create temp file");
    tf.write_all(contents).expect("write temp file");
    // Ensure contents are flushed
    tf.flush().expect("flush temp file");
    tf
}

/// Create a temporary directory and return it along with a child path inside it
/// for a filename the caller may create later. The TempDir keeps the directory
/// and any created children alive until dropped.
pub fn temp_dir_with_child(prefix: &str, child_filename: &str) -> (tempfile::TempDir, PathBuf) {
    let td = tempfile::Builder::new()
        .prefix(prefix)
        .tempdir()
        .expect("create temp dir");
    let p = td.path().join(child_filename);
    (td, p)
}

/// Convenience helper that creates a TempDir and writes `contents` to a child file.
/// Returns the TempDir and the child path. Both are valid until the TempDir is dropped.
pub fn temp_dir_with_file(prefix: &str, child_filename: &str, contents: &[u8]) -> (tempfile::TempDir, PathBuf) {
    let (td, p) = temp_dir_with_child(prefix, child_filename);
    std::fs::write(&p, contents).expect("write child file");
    (td, p)
}
