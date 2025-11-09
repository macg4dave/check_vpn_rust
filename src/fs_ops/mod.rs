use anyhow::{Context, Result};
use std::fs::{self, File};
use std::path::Path;

/// Small shared filesystem helpers used by multiple IO modules.
///
/// These wrap `std::fs` operations to provide consistent `anyhow::Context`
/// messages that include the file path and the operation kind ("json", "xml",
/// etc.). Keeping these helpers in one place avoids duplication between
/// `json_io::fs` and `xml_io::fs`.
pub fn read_to_string<P: AsRef<Path>>(path: P, kind: &str) -> Result<String> {
    let p = path.as_ref();
    fs::read_to_string(p).with_context(|| format!("failed to read {} file: {}", kind, p.display()))
}

pub fn write_string<P: AsRef<Path>>(path: P, contents: &str, kind: &str) -> Result<()> {
    let p = path.as_ref();
    fs::write(p, contents).with_context(|| format!("failed to write {} file: {}", kind, p.display()))
}

/// Convenience helper used when a caller already has an owned `String` and
/// wants to log the write size in bytes. Returns the number of bytes written
/// on success.
pub fn write_string_with_len<P: AsRef<Path>>(path: P, contents: &str, kind: &str) -> Result<usize> {
    write_string(path.as_ref(), contents, kind)?;
    Ok(contents.len())
}

/// Open a file for read and return a `File` with helpful context on failure.
pub fn open_file_for_read<P: AsRef<Path>>(path: P, kind: &str) -> Result<File> {
    let p = path.as_ref();
    File::open(p).with_context(|| format!("failed to open {} file: {}", kind, p.display()))
}

/// Create a file for write (truncating) and return a `File` with helpful context on failure.
pub fn create_file_for_write<P: AsRef<Path>>(path: P, kind: &str) -> Result<File> {
    let p = path.as_ref();
    File::create(p).with_context(|| format!("failed to create {} file: {}", kind, p.display()))
}

/// Return the metadata for a path with helpful context.
pub fn metadata<P: AsRef<Path>>(path: P, kind: &str) -> Result<std::fs::Metadata> {
    let p = path.as_ref();
    p.metadata().with_context(|| format!("failed to stat {} file: {}", kind, p.display()))
}

/// Ensure the parent directory for `path` exists, creating it (and parents)
/// if necessary. Provides contextual error messages mentioning `kind`.
#[allow(clippy::collapsible_if)]
pub fn ensure_parent_dir_exists<P: AsRef<Path>>(path: P, kind: &str) -> Result<()> {
    let p = path.as_ref();
    if let Some(parent) = p.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create parent dir for {}: {}", kind, parent.display()))?;
        }
    }
    Ok(())
}

/// Atomically write `contents` to `path` by writing to a temporary file in the
/// same directory and then renaming into place. Returns number of bytes
/// written on success. The `kind` parameter is used for error context.
pub fn atomic_write<P: AsRef<Path>>(path: P, contents: &str, kind: &str) -> Result<usize> {
    let p = path.as_ref();
    // Ensure parent dir exists first
    ensure_parent_dir_exists(p, kind)?;

    // Build a unique temporary filename in the same directory to avoid cross-device issues
    let file_name = p
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tmpfile".to_string());
    let unique = format!("{}.{}.{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0u128), file_name);
    let tmp_name = p.with_file_name(format!(".{}.tmp", unique));

    // Write to the temp file
    std::fs::write(&tmp_name, contents)
        .with_context(|| format!("failed to write temporary {} file: {}", kind, tmp_name.display()))?;

    // Move into place (atomic on most platforms when on same filesystem)
    std::fs::rename(&tmp_name, p)
        .with_context(|| format!("failed to rename temporary {} file to final path: {} -> {}", kind, tmp_name.display(), p.display()))?;

    Ok(contents.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_parent_and_metadata() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("file.txt");
        // parent shouldn't exist yet
        assert!(!nested.parent().unwrap().exists());

    ensure_parent_dir_exists(&nested, "test").unwrap();
        assert!(nested.parent().unwrap().exists());

        // create the file and check metadata
        std::fs::write(&nested, b"hello").unwrap();
        let meta = metadata(&nested, "test").unwrap();
        assert!(meta.is_file());
        assert_eq!(meta.len(), 5);
    }

    #[test]
    fn test_atomic_write_and_overwrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("out.txt");

        let n = atomic_write(&path, "first", "test").unwrap();
        assert_eq!(n, 5);
        let got = std::fs::read_to_string(&path).unwrap();
        assert_eq!(got, "first");

        let n2 = atomic_write(&path, "second content", "test").unwrap();
        assert_eq!(n2, "second content".len());
        let got2 = std::fs::read_to_string(&path).unwrap();
        assert_eq!(got2, "second content");
    }

    #[test]
    fn test_open_and_create_file_helpers() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("stream.txt");

        // create file for write and write to it
        {
            let mut f = create_file_for_write(&path, "test").unwrap();
            use std::io::Write;
            f.write_all(b"abc").unwrap();
        }

        // open for read and read contents
        let mut r = open_file_for_read(&path, "test").unwrap();
        let mut s = String::new();
        r.read_to_string(&mut s).unwrap();
        assert_eq!(s, "abc");
    }
}
