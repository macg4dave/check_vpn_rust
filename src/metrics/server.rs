use log::{error, info, trace};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crate::metrics::{build_health_response, build_metrics_response, build_not_found_response};

/// Start a tiny, dependency-free metrics HTTP server on the provided `addr`.
///
/// The server is intentionally minimal: it answers GET /health and GET /metrics
/// and runs until `keep_running` is set to false. Returns a JoinHandle for the
/// spawned thread so callers can join on shutdown. The function returns None
/// when binding the listener fails.
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
                    match stream.read(&mut buf) {
                        Ok(n) if n > 0 => {
                            let req = String::from_utf8_lossy(&buf[..n]);
                            trace!("metrics req: {}", req);
                            if req.starts_with("GET /health") {
                                let _ = stream.write_all(&build_health_response());
                            } else if req.starts_with("GET /metrics") {
                                let _ = stream.write_all(&build_metrics_response());
                            } else {
                                let _ = stream.write_all(&build_not_found_response());
                            }
                        }
                        _ => { /* ignore read errors or zero-length reads */ }
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
