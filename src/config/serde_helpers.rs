use serde::Serializer;

/// Ensure Option<bool> serializes as <tag>true</tag> / <tag>false</tag> when Some,
/// and is omitted entirely when None (to avoid empty elements like <tag/> that
/// quick-xml cannot deserialize back into a bool).
pub fn serialize_option_bool<S>(v: &Option<bool>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match v {
        Some(b) => s.serialize_bool(*b),
        None => s.serialize_none(),
    }
}
