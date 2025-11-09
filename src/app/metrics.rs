use log::{error, info};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

/// Start a tiny, dependency-free metrics HTTP server on the provided `addr`.
///
/// The server is minimal by design: it accepts connections on `addr` and
/// responds to GET /health and GET /metrics. The function returns a JoinHandle
/// which will run until `keep_running` is set to false. The caller may wake the
/// listener (e.g., by connecting) to speed shutdown.
pub fn start_metrics_server(addr: &str, keep_running: Arc<AtomicBool>) -> Option<JoinHandle<()>> {
    let addr = addr.to_string();
    let kr = keep_running.clone();
    Some(std::thread::spawn(move || {
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                error!("metrics server failed to bind {}: {}", addr, e);
                return;
            }
        };

        // Non-blocking accept so we can poll the shutdown flag.
        listener.set_nonblocking(true).ok();
        info!("metrics server listening on {}", addr);

        while kr.load(std::sync::atomic::Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _peer)) => {
                    // Read a small request; sufficient for simple GETs.
                    let mut buf = [0u8; 2048];
                    if let Ok(n) = stream.read(&mut buf) {
                        let req = String::from_utf8_lossy(&buf[..n]);
                        if req.starts_with("GET /health") {
                            let resp = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok";
                            let _ = stream.write_all(resp.as_bytes());
                        } else if req.starts_with("GET /metrics") {
                            let body = "# HELP check_vpn_up 1 if the service is up\n# TYPE check_vpn_up gauge\ncheck_vpn_up 1\n";
                            let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
                            let _ = stream.write_all(resp.as_bytes());
                        } else {
                            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                            let _ = stream.write_all(resp.as_bytes());
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                    continue;
                }
                Err(_) => break,
            }
        }

        info!("metrics server at {} shutting down", addr);
    }))
}
