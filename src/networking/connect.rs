use super::NetworkingError;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;
use log::trace;

/// Resolve `addr` (may yield multiple SocketAddrs) and attempt to connect to
/// each address with the provided `timeout`. Returns Ok(true) if any address
/// connects successfully, Ok(false) if none connect, or Err on name-resolution
/// failure.
pub fn try_connect(addr: &str, timeout: Duration) -> Result<bool, NetworkingError> {
    match addr.to_socket_addrs() {
        Ok(addrs) => {
            for socket in addrs {
                trace!("Resolved {} -> {}", addr, socket);
                if try_connect_addr(&socket, timeout) {
                    return Ok(true);
                }
            }
            Ok(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    // A small timeout to keep tests fast; the unit tests use loopback to avoid
    // network flakiness.
    const TEST_TIMEOUT_MS: u64 = 500;

    #[test]
    fn try_connect_detects_local_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        let port = listener.local_addr().expect("local addr").port();

        let addr = format!("127.0.0.1:{}", port);
        let res = try_connect(&addr, Duration::from_millis(TEST_TIMEOUT_MS));
        assert!(matches!(res, Ok(true)), "expected Ok(true), got: {:?}", res);

        drop(listener);
    }

    #[test]
    fn try_connect_unreachable_returns_false() {
        // Use high loopback port unlikely to be in use on CI.
        let addr = "127.0.0.1:65000".to_string();
        let res = try_connect(&addr, Duration::from_millis(TEST_TIMEOUT_MS));
        assert!(matches!(res, Ok(false)), "expected Ok(false), got: {:?}", res);
    }

    #[test]
    fn try_connect_dns_error_returns_err() {
        // Use a syntactically-valid but (very likely) non-resolvable hostname.
        // This should trigger a name-resolution error rather than a TCP timeout.
        let addr = "nonexistent.invalid.tld:12345";
        let res = try_connect(addr, Duration::from_millis(TEST_TIMEOUT_MS));
        assert!(matches!(res, Err(super::NetworkingError::DnsResolve(_))), "expected DnsResolve, got: {:?}", res);
    }
}
