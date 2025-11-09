use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

/// Read a generic struct from a JSON file at `path`.
/// - Provides helpful error context for malformed JSON.
/// - Logs basic observability (duration, size) at debug level.
pub fn read_json<T: DeserializeOwned>(path: &str) -> Result<T> {
    let started = Instant::now();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read json file: {}", path))?;
    log::debug!(
        "json_io::read_json reading path={}, bytes={}",
        path,
        contents.len()
    );

    let parsed: T = serde_json::from_str(&contents).with_context(|| {
        format!(
            "failed to parse json; file='{}' ({} bytes). Ensure the document is well-formed and matches the expected schema.",
            path,
            contents.len()
        )
    })?;

    log::debug!(
        "json_io::read_json parsed OK path={} in {:?}",
        path,
        started.elapsed()
    );
    Ok(parsed)
}

/// Serialize `value` to JSON and write to `path` (overwrites existing file).
pub fn write_json<T: Serialize>(value: &T, path: &str) -> Result<()> {
    let started = Instant::now();
    // When `json_pretty` feature is enabled, write pretty-printed JSON.
    #[cfg(feature = "json_pretty")]
    let json = serde_json::to_string_pretty(value).context("failed to serialize to json")?;
    #[cfg(not(feature = "json_pretty"))]
    let json = serde_json::to_string(value).context("failed to serialize to json")?;

    fs::write(path, &json).with_context(|| format!("failed to write json file: {}", path))?;
    log::debug!(
        "json_io::write_json wrote path={} bytes={} in {:?}",
        path,
        json.len(),
        started.elapsed()
    );
    Ok(())
}

/// Read a value from any reader using streaming deserialization.
pub fn read_json_from_reader<R: Read, T: DeserializeOwned>(rdr: R) -> Result<T> {
    let started = Instant::now();
    let parsed = serde_json::from_reader(rdr).context("failed to deserialize json from reader")?;
    log::debug!("json_io::read_json_from_reader done in {:?}", started.elapsed());
    Ok(parsed)
}

/// Write a value to any writer using streaming serialization.
pub fn write_json_to_writer<W: Write, T: Serialize>(value: &T, mut wr: W) -> Result<()> {
    let started = Instant::now();
    #[cfg(feature = "json_pretty")]
    serde_json::to_writer_pretty(&mut wr, value).context("failed to serialize json to writer")?;
    #[cfg(not(feature = "json_pretty"))]
    serde_json::to_writer(&mut wr, value).context("failed to serialize json to writer")?;
    log::debug!("json_io::write_json_to_writer wrote in {:?}", started.elapsed());
    Ok(())
}

/// Read a value from a file using streaming deserialization.
pub fn read_json_from_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let path_ref = path.as_ref();
    let f = File::open(path_ref).with_context(|| format!("failed to open json file: {}", path_ref.display()))?;
    read_json_from_reader(f)
}

/// Write a value to a file using streaming serialization (overwrites existing file).
pub fn write_json_to_file<T: Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<()> {
    let path_ref = path.as_ref();
    let f = File::create(path_ref).with_context(|| format!("failed to create json file: {}", path_ref.display()))?;
    write_json_to_writer(value, f)
}
