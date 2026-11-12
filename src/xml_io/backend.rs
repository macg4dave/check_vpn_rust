use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

// XML support is unconditional; formatting remains feature-controlled.

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

#[cfg(not(feature = "xml_pretty"))]
pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    Ok(quick_xml::se::to_string(value)?)
}

#[cfg(feature = "xml_pretty")]
pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    let compact = quick_xml::se::to_string(value)?;
    Ok(pretty_print_xml(&compact))
}

fn extract_line_col(_err_msg: &str, doc: &str) -> Option<(usize, usize)> {
    if !doc.trim_end().ends_with('>') {
        return Some((doc.lines().count(), 1));
    }
    None
}

#[cfg(feature = "xml_pretty")]
fn pretty_print_xml(s: &str) -> String {
    // Lightweight formatter for simple config XML (best-effort indentation).
    let mut out = String::new();
    let mut indent: i32 = 0;
    let tokens = s.replace("><", ">\n<");
    for line in tokens.lines() {
        let l = line.trim();
        if l.starts_with("</") && indent > 0 {
            indent -= 1;
        }
        for _ in 0..indent {
            out.push_str("  ");
        }
        out.push_str(l);
        out.push('\n');
        if l.starts_with('<') && !l.starts_with("</") && !l.ends_with("/>") && !l.contains("</") {
            indent += 1;
        }
    }
    out.trim_end().to_string()
}
