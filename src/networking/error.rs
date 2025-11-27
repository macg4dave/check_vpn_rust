use thiserror::Error;

/// Networking-specific errors returned by connectivity helpers.
///
/// This lives in its own module so the type can be expanded or split into
/// sub-modules later without causing large diffs in the parent `networking`
/// module. The error variants intentionally store `String` payloads to avoid
/// binding the public API to concrete external error types.
#[derive(Debug, Error)]
pub enum NetworkingError {
    /// DNS or name resolution failed for the provided address (original error string)
    #[error("DNS resolution failed: {0}")]
    DnsResolve(String),
    /// Generic I/O error (propagated from underlying socket ops)
    #[error("I/O error: {0}")]
    Io(String),
}
