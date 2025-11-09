use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;

/// Read a generic struct from an XML file at `path` using `quick-xml` serde support.
pub fn read_xml<T: DeserializeOwned>(path: &str) -> Result<T> {
    let contents = fs::read_to_string(path).with_context(|| format!("failed to read xml file: {}", path))?;
    // Use quick_xml for parsing which is faster and has more active maintenance.
    let parsed: T = quick_xml::de::from_str(&contents).context("failed to parse xml")?;
    Ok(parsed)
}

/// Serialize `value` to XML and write to `path` (overwrites existing file).
pub fn write_xml<T: Serialize>(value: &T, path: &str) -> Result<()> {
    let xml = quick_xml::se::to_string(value).context("failed to serialize to xml")?;
    fs::write(path, xml).with_context(|| format!("failed to write xml file: {}", path))?;
    Ok(())
}
