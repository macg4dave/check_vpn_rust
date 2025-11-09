use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Read;
use std::io::Write;
use std::time::Instant;

/// Deserialize a JSON value from any reader using Serde's streaming API.
///
/// This is useful when you already have an opened file, network stream or an
/// in-memory reader and want to avoid intermediate String allocations.
pub fn read_json_from_reader<R: Read, T: DeserializeOwned>(rdr: R) -> Result<T> {
    let started = Instant::now();
    let parsed = serde_json::from_reader(rdr).context("failed to deserialize json from reader")?;
    log::debug!("json_io::read_json_from_reader done in {:?}", started.elapsed());
    Ok(parsed)
}

/// Serialize a value and write it to any writer using Serde's streaming API.
pub fn write_json_to_writer<W: Write, T: Serialize>(value: &T, mut wr: W) -> Result<()> {
    let started = Instant::now();
    #[cfg(feature = "json_pretty")]
    serde_json::to_writer_pretty(&mut wr, value).context("failed to serialize json to writer")?;
    #[cfg(not(feature = "json_pretty"))]
    serde_json::to_writer(&mut wr, value).context("failed to serialize json to writer")?;
    log::debug!("json_io::write_json_to_writer wrote in {:?}", started.elapsed());
    Ok(())
}
