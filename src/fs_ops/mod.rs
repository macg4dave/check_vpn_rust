use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Small shared filesystem helpers used by multiple IO modules.
///
/// These wrap `std::fs` operations to provide consistent `anyhow::Context`
/// messages that include the file path and the operation kind.
pub fn read_to_string<P: AsRef<Path>>(path: P, kind: &str) -> Result<String> {
    let p = path.as_ref();
    fs::read_to_string(p).with_context(|| format!("failed to read {} file: {}", kind, p.display()))
}

pub fn write_string<P: AsRef<Path>>(path: P, contents: &str, kind: &str) -> Result<()> {
    let p = path.as_ref();
    fs::write(p, contents)
        .with_context(|| format!("failed to write {} file: {}", kind, p.display()))
}

/// Ensure the parent directory for `path` exists, creating it (and parents)
/// if necessary. Provides contextual error messages mentioning `kind`.
pub fn ensure_parent_dir_exists<P: AsRef<Path>>(path: P, kind: &str) -> Result<()> {
    let p = path.as_ref();
    match p.parent() {
        Some(parent) if !parent.exists() => {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create parent dir for {}: {}",
                    kind,
                    parent.display()
                )
            })?;
        }
        _ => {}
    }
    Ok(())
}
