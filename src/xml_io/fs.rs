use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Instant;

/// Read a generic struct from an XML file at `path` using the selected XML backend.
/// Provides helpful error context and debug logging.
pub fn read_xml<T: DeserializeOwned>(path: &str) -> Result<T> {
    let started = Instant::now();
    let contents = crate::fs_ops::read_to_string(path, "xml")?;
    log::debug!(
        "xml_io::read_xml reading path={}, bytes={}",
        path,
        contents.len()
    );
    #[cfg(feature = "xml_tracing")]
    let span = tracing::info_span!("xml_read", path, size = contents.len());
    #[cfg(feature = "xml_tracing")]
    let _enter = span.enter();

    let parsed: T = match super::backend::deserialize_xml(&contents) {
        Ok(v) => v,
        Err(e) => {
            return Err(e.context(format!(
                "failed to parse xml; file='{}' ({} bytes). Ensure the document is well-formed and matches the expected schema.",
                path,
                contents.len()
            )));
        }
    };

    log::debug!(
        "xml_io::read_xml parsed OK path={} in {:?}",
        path,
        started.elapsed()
    );
    Ok(parsed)
}

/// Serialize `value` to XML and write to `path` (overwrites existing file).
pub fn write_xml<T: Serialize>(value: &T, path: &str) -> Result<()> {
    let started = Instant::now();
    #[cfg(feature = "xml_tracing")]
    let span = tracing::info_span!("xml_write", path);
    #[cfg(feature = "xml_tracing")]
    let _enter = span.enter();

    let xml = super::backend::serialize_xml(value).context("failed to serialize to xml")?;
    crate::fs_ops::write_string(path, &xml, "xml")?;
    log::debug!(
        "xml_io::write_xml wrote path={} bytes={} in {:?}",
        path,
        xml.len(),
        started.elapsed()
    );
    Ok(())
}
