use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::thread::sleep;
use std::fmt;
use std::error::Error;
use log::{debug, trace};

/// Default timeout (seconds) for connectivity checks.
pub const DEFAULT_TIMEOUT_SECS: u64 = 2;

/// Default number of retries for connectivity checks.
pub const DEFAULT_RETRIES: usize = 1;

/// Default ports to try when endpoint does not include an explicit port.
pub const DEFAULT_PORTS: [u16; 3] = [443u16, 53u16, 80u16];

/// Check whether any of the provided endpoints are reachable using a TCP connect
/// as a lightweight "ping" (avoids raw socket privileges).
///
/// Backwards-compatible simple API: `is_online(endpoints, timeout_secs)` uses
/// `DEFAULT_PORTS` for addresses without an explicit port.
/// Networking-specific errors returned from connectivity checks.
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

/// Backwards-compatible simple API: `is_online(endpoints, timeout_secs)` uses
/// `DEFAULT_PORTS` for addresses without an explicit port.
pub fn is_online<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64) -> std::result::Result<bool, NetworkingError> {
    // Backwards-compatible: single attempt (no retries)
    is_online_with_retries(endpoints, timeout_secs, &DEFAULT_PORTS, 1)
}

/// More flexible API which accepts a slice of ports to try when an endpoint
/// doesn't include a port. Use this when you need to tighten or change the
/// port list at runtime.
pub fn is_online_with_ports<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16]) -> std::result::Result<bool, NetworkingError> {
    is_online_with_retries(endpoints, timeout_secs, ports, 1)
}

/// Like `is_online_with_ports` but retry `retries` times with a small backoff
/// between attempts. This helps against transient network races or services
/// that appear shortly after the check begins.
pub fn is_online_with_retries<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16], retries: usize) -> std::result::Result<bool, NetworkingError> {
    let timeout = Duration::from_secs(timeout_secs);

    for ep in endpoints {
        let s = ep.as_ref();

        // If s already contains a port, try to connect directly.
        if s.contains(":") {
            for attempt in 0..retries.max(1) {
                debug!("Attempting connect to {} (attempt {}/{})", s, attempt + 1, retries.max(1));
                match try_connect(s, timeout) {
                    Ok(true) => return Ok(true),
                    Ok(false) => {
                        // connection attempt failed; will retry if configured
                    }
                    Err(e) => return Err(e),
                }

                if attempt + 1 < retries {
                    // Backoff (linear): 200ms * (attempt+1)
                    sleep(Duration::from_millis(200 * (attempt as u64 + 1)));
                }
            }
            continue;
        }

        // Try configured ports when no port specified.
        for &port in ports {
            let addr = format!("{}:{}", s, port);
            for attempt in 0..retries.max(1) {
                debug!("Attempting connect to {} (attempt {}/{})", addr, attempt + 1, retries.max(1));
                match try_connect(&addr, timeout) {
                    Ok(true) => return Ok(true),
                    Ok(false) => {
                        // not reachable on this attempt/port
                    }
                    Err(e) => return Err(e),
                }

                if attempt + 1 < retries {
                    sleep(Duration::from_millis(200 * (attempt as u64 + 1)));
                }
            }
        }
    }

    Ok(false)
}

fn try_connect(addr: &str, timeout: Duration) -> std::result::Result<bool, NetworkingError> {
    // Resolve the address (may return multiple socket addrs) and try each.
    match addr.to_socket_addrs() {
        Ok(addrs) => {
            let mut any_success = false;
            for socket in addrs {
                trace!("Resolved {} -> {}", addr, socket);
                if try_connect_addr(&socket, timeout) {
                    any_success = true;
                    break;
                }
            }
            Ok(any_success)
        }
        Err(e) => Err(NetworkingError::DnsResolve(e.to_string())),
    }
}

fn try_connect_addr(socket: &SocketAddr, timeout: Duration) -> bool {
    match TcpStream::connect_timeout(socket, timeout) {
        Ok(_) => true,
        Err(e) => {
            trace!("connect to {} failed: {}", socket, e);
            false
        }
    }
}
