use anyhow::Result;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::thread::sleep;

/// Default timeout (seconds) for connectivity checks.
pub const DEFAULT_TIMEOUT_SECS: u64 = 2;

/// Default ports to try when endpoint does not include an explicit port.
pub const DEFAULT_PORTS: [u16; 3] = [443u16, 53u16, 80u16];

/// Check whether any of the provided endpoints are reachable using a TCP connect
/// as a lightweight "ping" (avoids raw socket privileges).
///
/// Backwards-compatible simple API: `is_online(endpoints, timeout_secs)` uses
/// `DEFAULT_PORTS` for addresses without an explicit port.
pub fn is_online<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64) -> Result<bool> {
    // Backwards-compatible: single attempt (no retries)
    is_online_with_retries(endpoints, timeout_secs, &DEFAULT_PORTS, 1)
}

/// More flexible API which accepts a slice of ports to try when an endpoint
/// doesn't include a port. Use this when you need to tighten or change the
/// port list at runtime.
pub fn is_online_with_ports<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16]) -> Result<bool> {
    is_online_with_retries(endpoints, timeout_secs, ports, 1)
}

/// Like `is_online_with_ports` but retry `retries` times with a small backoff
/// between attempts. This helps against transient network races or services
/// that appear shortly after the check begins.
pub fn is_online_with_retries<S: AsRef<str>>(endpoints: &[S], timeout_secs: u64, ports: &[u16], retries: usize) -> Result<bool> {
    let timeout = Duration::from_secs(timeout_secs);

    for ep in endpoints {
        let s = ep.as_ref();

        // If s already contains a port, try to connect directly.
        if s.contains(":") {
            for attempt in 0..retries.max(1) {
                if try_connect(s, timeout) {
                    return Ok(true);
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
                if try_connect(&addr, timeout) {
                    return Ok(true);
                }
                if attempt + 1 < retries {
                    sleep(Duration::from_millis(200 * (attempt as u64 + 1)));
                }
            }
        }
    }

    Ok(false)
}

fn try_connect(addr: &str, timeout: Duration) -> bool {
    // Resolve the address (may return multiple socket addrs) and try each.
    match addr.to_socket_addrs() {
        Ok(addrs) => {
            for socket in addrs {
                if try_connect_addr(&socket, timeout) {
                    return true;
                }
            }
            false
        }
        Err(_) => false,
    }
}

fn try_connect_addr(socket: &SocketAddr, timeout: Duration) -> bool {
    TcpStream::connect_timeout(socket, timeout).is_ok()
}
