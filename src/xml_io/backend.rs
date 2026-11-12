use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn deserialize_xml<T: DeserializeOwned>(s: &str) -> Result<T> {
    // Preserve the parser's original error instead of guessing source locations.
    Ok(quick_xml::de::from_str(s)?)
}

pub fn serialize_xml<T: Serialize>(value: &T) -> Result<String> {
    let mut output = String::new();
    let mut serializer = quick_xml::se::Serializer::new(&mut output);
    if cfg!(feature = "xml_pretty") {
        serializer.indent(' ', 2);
    }
    value.serialize(serializer)?;
    Ok(output)
}
