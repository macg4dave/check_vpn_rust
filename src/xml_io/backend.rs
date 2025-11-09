use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

// Backend implementations selected by cargo feature flags.

#[cfg(feature = "xml_quick")]
pub fn deserialize_xml<T: DeserializeOwned>(s: &str) -> Result<T> {
    match quick_xml::de::from_str::<T>(s) {
        Ok(v) => Ok(v),
        Err(e) => {
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

#[cfg(all(feature = "xml_quick", not(feature = "xml_pretty")))]
pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    Ok(quick_xml::se::to_string(value)?)
}

#[cfg(all(feature = "xml_quick", feature = "xml_pretty"))]
pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    let compact = quick_xml::se::to_string(value)?;
    let pretty = compact.replace("><", ">
<");
    Ok(pretty)
}

#[cfg(feature = "xml_quick")]
fn extract_line_col(_err_msg: &str, doc: &str) -> Option<(usize, usize)> {
    if !doc.trim_end().ends_with('>') {
        return Some((doc.lines().count(), 1));
    }
    None
}

#[cfg(all(feature = "xml_serde", not(feature = "xml_quick")))]
pub fn deserialize_xml<T: DeserializeOwned>(s: &str) -> Result<T> {
    Ok(serde_xml_rs::from_str::<T>(s)?)
}

#[cfg(all(feature = "xml_serde", not(feature = "xml_quick")))]
pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_xml_rs::to_string(value)?)
}

#[cfg(all(not(feature = "xml_quick"), not(feature = "xml_serde")))]
compile_error!("Enable at least one XML backend feature: `xml_quick` (default) or `xml_serde`.");
