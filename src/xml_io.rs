use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::time::Instant;

/// Read a generic struct from an XML file at `path` using the selected XML backend.
/// - Provides helpful error context for malformed XML.
/// - Logs basic observability (duration, size) at debug level.
pub fn read_xml<T: DeserializeOwned>(path: &str) -> Result<T> {
    let started = Instant::now();
    let contents = fs::read_to_string(path)
        .with_context(|| format!("failed to read xml file: {}", path))?;
    log::debug!(
        "xml_io::read_xml reading path={}, bytes={}",
        path,
        contents.len()
    );
    #[cfg(feature = "xml_tracing")]
    let span = tracing::info_span!("xml_read", path, size = contents.len());
    #[cfg(feature = "xml_tracing")]
    let _enter = span.enter();

    let parsed: T = match deserialize_xml(&contents) {
        Ok(v) => v,
        Err(e) => {
            // enrich with size + path context
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
/// When the `xml_pretty` feature is enabled, the output is indented where supported.
pub fn write_xml<T: Serialize>(value: &T, path: &str) -> Result<()> {
    let started = Instant::now();
    #[cfg(feature = "xml_tracing")]
    let span = tracing::info_span!("xml_write", path);
    #[cfg(feature = "xml_tracing")]
    let _enter = span.enter();

    let xml = serialize_xml(value).context("failed to serialize to xml")?;
    fs::write(path, &xml).with_context(|| format!("failed to write xml file: {}", path))?;
    log::debug!(
        "xml_io::write_xml wrote path={} bytes={} in {:?}",
        path,
        xml.len(),
        started.elapsed()
    );
    Ok(())
}

// --- Backend selection ------------------------------------------------------

#[cfg(feature = "xml_quick")]
fn deserialize_xml<T: DeserializeOwned>(s: &str) -> Result<T> {
    use anyhow::anyhow;
    match quick_xml::de::from_str::<T>(s) {
        Ok(v) => Ok(v),
        Err(e) => {
            // Attempt to compute a line number heuristically if the error exposes a byte index pattern.
            let raw = e.to_string();
            let line_col_hint = extract_line_col(&raw, s);
            let msg = if let Some((line, col)) = line_col_hint {
                format!("{} (at line {}, column {})", raw, line, col)
            } else {
                raw
            };
            Err(anyhow!(msg))
        }
    }
}

// Compact serialization (no pretty feature)
#[cfg(all(feature = "xml_quick", not(feature = "xml_pretty")))]
fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    Ok(quick_xml::se::to_string(value)?)
}

// Very naive pretty-print: insert newlines between adjacent tags. Keeps stable behavior without
// depending on potentially unstable pretty APIs.
#[cfg(all(feature = "xml_quick", feature = "xml_pretty"))]
fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    let compact = quick_xml::se::to_string(value)?;
    let pretty = compact.replace("><", ">\n<");
    Ok(pretty)
}

#[cfg(feature = "xml_quick")]
fn extract_line_col(_err_msg: &str, doc: &str) -> Option<(usize, usize)> {
    // Fallback heuristic: just return first line/col (1,1) if the doc looks truncated malformed.
    // A more sophisticated approach would parse byte indices; kept simple for stability.
    if !doc.trim_end().ends_with('>') {
        return Some( (doc.lines().count(), 1) );
    }
    None
}

#[cfg(all(feature = "xml_serde", not(feature = "xml_quick")))]
fn deserialize_xml<T: DeserializeOwned>(s: &str) -> Result<T> {
    Ok(serde_xml_rs::from_str::<T>(s)?)
}

#[cfg(all(feature = "xml_serde", not(feature = "xml_quick")))]
fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_xml_rs::to_string(value)?)
}

#[cfg(all(not(feature = "xml_quick"), not(feature = "xml_serde")))]
compile_error!("Enable at least one XML backend feature: `xml_quick` (default) or `xml_serde`.");
