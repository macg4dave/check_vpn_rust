use std::error::Error;
use std::fmt;

/// Networking-specific errors returned by connectivity helpers.
///
/// This lives in its own module so the type can be expanded or split into
/// sub-modules later without causing large diffs in the parent `networking`
/// module. The error variants intentionally store `String` payloads to avoid
/// binding the public API to concrete external error types.
#[derive(Debug)]
pub enum NetworkingError {
    /// DNS or name resolution failed for the provided address (original error string)
    DnsResolve(String),
    /// Generic I/O error (propagated from underlying socket ops)
    Io(String),
}

impl fmt::Display for NetworkingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkingError::DnsResolve(s) => write!(f, "DNS resolution failed: {}", s),
            NetworkingError::Io(s) => write!(f, "I/O error: {}", s),
        }
    }
}

impl Error for NetworkingError {}
