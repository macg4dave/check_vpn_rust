use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Start a minimal metrics server bound to `addr` (e.g. "127.0.0.1:9090").
///
/// This is intentionally a tiny, dependency-free stub that listens on the
/// provided address and drains connections until `keep_running` is cleared.
/// It returns a `JoinHandle` so the caller can join the thread on shutdown.
pub fn start_metrics_server(addr: &str, keep_running: Arc<AtomicBool>) -> Option<JoinHandle<()>> {
    let bind_addr = addr.replace("http://", "");
    // Attempt to bind; if we can't bind (port in use / permissions), just skip metrics.
    let listener = match TcpListener::bind(&bind_addr) {
        Ok(l) => l,
        Err(_) => return None,
    };

    // Set non-blocking so we can poll the keep_running flag.
    let _ = listener.set_nonblocking(true);

    let handle = thread::spawn(move || {
        while keep_running.load(std::sync::atomic::Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    // Minimal response: ignore content and close.
                    let _ = stream.set_nonblocking(true);
                    // Sleep briefly to yield.
                    thread::sleep(Duration::from_millis(10));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    Some(handle)
}
