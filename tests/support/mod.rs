#![allow(dead_code)]
// Lightweight helpers for integration tests.
// Keep the API small and std-only so tests don't pull extra deps.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Scoped env helper for tests. Setting environment variables in tests can cause
/// races between parallel tests and some historical targets required `unsafe`
/// blocks for env mutation. We encapsulate the mutation and restoration here so
/// tests don't need their own unsafe blocks and are clearer about intent.
use std::ffi::OsString;

/// A small RAII guard that sets an environment variable for the duration of the
/// guard and restores the previous value (or removes it) on drop.
pub struct ScopedEnv {
    key: String,
    prev: Option<OsString>,
    removed: bool,
}

impl ScopedEnv {
    /// Set `key` to `value` for the lifetime of the returned guard.
    /// If `value` is None the variable is removed for the lifetime.
    pub fn set<K: Into<String>, V: Into<Option<OsString>>>(key: K, value: V) -> ScopedEnv {
        let key_s = key.into();
        let prev = std::env::var_os(&key_s);
        match value.into() {
            Some(v) => std::env::set_var(&key_s, &v),
            None => std::env::remove_var(&key_s),
        }
        ScopedEnv {
            key: key_s,
            prev,
            removed: false,
        }
    }

    /// Convenience to set a string value.
    pub fn set_str(key: &str, value: &str) -> ScopedEnv {
        ScopedEnv::set(key, Some(OsString::from(value)))
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        if let Some(prev) = self.prev.take() {
            std::env::set_var(&self.key, prev);
        } else if !self.removed {
            // No previous value: remove it if still present
            std::env::remove_var(&self.key);
        }
    }
}

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
        if let Some(tx) = self.shutdown.take() {
            // best-effort shutdown
            let _ = tx.send(());
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

/// Start a simple HTTP server that responds with `body` and HTTP status `status`.
/// Returns a `TestHttpServer` which can be stopped by calling `stop()` or dropping it.
pub fn start_http_server(body: &'static str, status: u16) -> TestHttpServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().expect("local addr").to_string();

    let (tx, rx) = channel::<()>();
    // Use an Arc<Mutex<..>> for the body so it's cheaply shareable if handlers read it
    let body_arc = Arc::new(body.to_string());

    // Set non-blocking accept so shutdown signals are seen promptly
    listener
        .set_nonblocking(true)
        .expect("failed to set nonblocking listener");
    let handle = thread::spawn(move || {
        // Run until shutdown is requested via rx; use non-blocking accept in a loop
        loop {
            if rx.try_recv().is_ok() {
                break;
            }
            match listener.accept() {
                Ok((mut s, _)) => {
                    // Read the request (rudimentary) until blank line
                    let mut buf = [0u8; 1024];
                    match s.read(&mut buf) {
                        Ok(_) => {}
                        Err(_) => continue,
                    }

                    // Construct a simple HTTP response
                    let body = body_arc.as_str();
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
                Err(_) => continue,
            }
        }
    });

    TestHttpServer {
        addr,
        shutdown: Some(tx),
        handle: Some(handle),
    }
}
