use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::Path;
use std::time::Instant;

/// Read a generic struct from a JSON file at `path` using a simple
/// read-to-string then deserialize approach. This provides helpful
/// error context including the file path.
pub fn read_json<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let path_ref = path.as_ref();
    let started = Instant::now();
    let contents = crate::fs_ops::read_to_string(path_ref, "json")?;

    log::debug!(
        "json_io::read_json reading path={} bytes={}",
        path_ref.display(),
        contents.len()
    );

    let parsed: T = serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse json; file='{}' ({} bytes). Ensure the document is well-formed and matches the expected schema.",
            path_ref.display(),
            contents.len()
        )
    })?;

    log::debug!(
        "json_io::read_json parsed OK path={} in {:?}",
        path_ref.display(),
        started.elapsed()
    );
    Ok(parsed)
}

/// Serialize `value` to JSON and write to `path` (overwrites existing file).
pub fn write_json<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let started = Instant::now();
    #[cfg(feature = "json_pretty")]
    let json = serde_json::to_string_pretty(value).context("failed to serialize to json")?;
    #[cfg(not(feature = "json_pretty"))]
    let json = serde_json::to_string(value).context("failed to serialize to json")?;

    crate::fs_ops::write_string(path_ref, &json, "json")?;
    log::debug!(
        "json_io::write_json wrote path={} bytes={} in {:?}",
        path_ref.display(),
        json.len(),
        started.elapsed()
    );
    Ok(())
}

/// Read a value from a file using streaming deserialization.
pub fn read_json_from_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let path_ref = path.as_ref();
    let f = crate::fs_ops::open_file_for_read(path_ref, "json")?;
    crate::json_io::stream::read_json_from_reader(f)
}

/// Write a value to a file using streaming serialization (overwrites existing file).
pub fn write_json_to_file<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let f = crate::fs_ops::create_file_for_write(path_ref, "json")?;
    crate::json_io::stream::write_json_to_writer(value, f)
}
