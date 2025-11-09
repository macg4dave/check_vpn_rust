use std::time::Duration;
use std::thread::sleep;
use log::{debug, trace};
mod connect;
mod error;
pub use error::NetworkingError;

/// Default timeout (seconds) for connectivity checks.
pub const DEFAULT_TIMEOUT_SECS: u64 = 2;

/// Default number of retries for connectivity checks.
pub const DEFAULT_RETRIES: usize = 1;

/// Default ports to try when endpoint does not include an explicit port.
pub const DEFAULT_PORTS: [u16; 3] = [443u16, 53u16, 80u16];

/// Check whether any of the provided endpoints are reachable using a TCP connect
/// as a lightweight "ping" (avoids raw socket privileges).
/// Backwards-compatible simple API: `is_online(endpoints, timeout_secs)` uses
/// `DEFAULT_PORTS` for addresses without an explicit port.
// Networking-specific errors are defined in `networking::error` and
// re-exported above as `networking::NetworkingError` for backwards
// compatibility. See `src/networking/error.rs` for details.
pub fn is_online<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64) -> Result<bool, NetworkingError> {
    // Backwards-compatible: single attempt (no retries)
    is_online_with_retries(endpoints, timeout_secs, &DEFAULT_PORTS, 1)
}

/// More flexible API which accepts a slice of ports to try when an endpoint
/// doesn't include a port. Use this when you need to tighten or change the
/// port list at runtime.
pub fn is_online_with_ports<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16]) -> Result<bool, NetworkingError> {
    is_online_with_retries(endpoints, timeout_secs, ports, 1)
}

/// Like `is_online_with_ports` but retry `retries` times with a small backoff
/// between attempts. This helps against transient network races or services
/// that appear shortly after the check begins.
pub fn is_online_with_retries<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16], retries: usize) -> Result<bool, NetworkingError> {
    let timeout = Duration::from_secs(timeout_secs);
    // Ensure at least one attempt is performed; keeps loop logic simple.
    let attempts = retries.max(1);

    // For each endpoint, produce the candidate address strings to try.
    // If the endpoint already contains a port ("host:port"), use it directly;
    // otherwise map it against the supplied ports slice.
    for ep in endpoints {
        let s = ep.as_ref();

        let candidates = if s.contains(':') {
            // Already contains a port
            vec![s.to_string()]
        } else {
            ports.iter().map(|p| format!("{}:{}", s, p)).collect()
        };

        // Try each candidate address; each may be attempted `attempts` times
        // to allow transient failures to recover.
        for addr in candidates {
            for attempt in 1..=attempts {
                debug!("Attempting connect to {} (attempt {}/{})", addr, attempt, attempts);
                match connect::try_connect(&addr, timeout) {
                    Ok(true) => return Ok(true),
                    Ok(false) => {
                        // not reachable right now; try again if attempts remain
                    }
                    Err(e) => {
                        // Name resolution or other networking error: treat as an
                        // immediate failure per crate semantics (bubble up).
                        return Err(e);
                    }
                }

                if attempt < attempts {
                    // Linear backoff: 200ms * attempt_index (1-based)
                    let backoff_ms = 200u64 * (attempt as u64);
                    trace!("backoff {}ms before next attempt", backoff_ms);
                    sleep(Duration::from_millis(backoff_ms));
                }
            }
        }
    }

    // No endpoint accepted a connection within given attempts/timeouts.
    Ok(false)
}

