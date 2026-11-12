#![allow(dead_code)]
// Lightweight helpers for integration tests.
// Keep the API small and std-only so tests don't pull extra deps.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Sender, TryRecvError};
use std::thread;
use std::time::Duration;

// Re-export smaller helper modules so tests import the single `support` module.
pub mod env_helpers;
pub mod temp_helpers;
#[allow(unused_imports)]
pub use env_helpers::*;
#[allow(unused_imports)]
pub use temp_helpers::*;

/// Bind an ephemeral TCP listener on localhost and return it plus its port.
///
/// Tests can hold the returned listener to ensure the chosen port remains
/// bound for the duration of the test (useful for connectivity checks).
pub fn tcp_listener_and_port() -> (TcpListener, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let port = listener.local_addr().expect("local addr").port();
    (listener, port)
}

/// Find a free local port without holding the listener open. Useful when a
/// test only needs the port number for a short-lived server created later.
pub fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    listener.local_addr().expect("local addr").port()
}

/// A tiny HTTP server for tests. Use `start_http_server` to create one.
pub struct TestHttpServer {
    pub addr: String,
    shutdown: Option<Sender<()>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl TestHttpServer {
    /// Return a full URL for the server (http://127.0.0.1:<port>/)
    pub fn url(&self) -> String {
        format!("http://{}/", self.addr)
    }

    /// Stop the server and wait for the thread to exit.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle.take() {
            handle.join().expect("HTTP test server panicked");
        }
    }
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Start a simple HTTP server that responds with `body` and HTTP status `status`.
/// Returns a `TestHttpServer` which can be stopped by calling `stop()` or dropping it.
pub fn start_http_server(body: &'static str, status: u16) -> TestHttpServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().expect("local addr").to_string();

    let (tx, rx) = channel::<()>();

    // Set non-blocking accept so shutdown signals are seen promptly
    listener
        .set_nonblocking(true)
        .expect("failed to set nonblocking listener");
    let handle = thread::spawn(move || {
        // Run until shutdown is requested via rx; use non-blocking accept in a loop
        loop {
            if !matches!(rx.try_recv(), Err(TryRecvError::Empty)) {
                break;
            }
            match listener.accept() {
                Ok((mut s, _)) => {
                    // Bound reads so a stalled client cannot prevent server shutdown.
                    s.set_read_timeout(Some(Duration::from_millis(200)))
                        .expect("set read timeout");
                    s.set_write_timeout(Some(Duration::from_millis(200)))
                        .expect("set write timeout");
                    let mut buf = [0u8; 1024];
                    match s.read(&mut buf) {
                        Ok(0) => continue,
                        Ok(_) => {}
                        Err(_) => continue,
                    }

                    // Construct a simple HTTP response
                    let resp = format!(
                        "HTTP/1.1 {} OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
                        status,
                        body.len(),
                        body
                    );
                    let _ = s.write_all(resp.as_bytes());
                    let _ = s.flush();
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No incoming connection, check for shutdown again and sleep briefly
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                Err(_) => break,
            }
        }
    });

    TestHttpServer {
        addr,
        shutdown: Some(tx),
        handle: Some(handle),
    }
}
