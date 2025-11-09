use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::thread;
use std::time::Duration;

// A tiny controllable HTTP server used in tests: returns 500 for the first N
// requests, then 200 with a JSON body. It supports graceful shutdown to avoid
// hanging the test when the client gives up before the success response.
struct SequenceServer {
    url: String,
    stop: Arc<AtomicBool>,
    handle: thread::JoinHandle<()>,
}

impl SequenceServer {
    fn start(failures: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        // Non-blocking to allow periodic stop checks.
        listener.set_nonblocking(true).expect("set nonblocking");
        let addr = listener.local_addr().unwrap();
        let url = format!("http://{}:{}{}", addr.ip(), addr.port(), "/json");

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            // Accept in a loop with non-blocking accept and short sleeps so we
            // can observe the shutdown flag promptly.
            loop {
                if stop_clone.load(Ordering::SeqCst) {
                    break;
                }

                match listener.accept() {
                    Ok((mut stream, _peer)) => {
                        // Read the request headers (simplistic, up to double CRLF)
                        let mut buf = [0u8; 1024];
                        let _ = stream.read(&mut buf); // ignore parse

                        let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1; // 1-based
                        if attempt <= failures {
                            let resp = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                            let _ = stream.write_all(resp.as_bytes());
                            // close connection (drop) to force new connect on retry
                        } else {
                            let body = "{\"isp\":\"Sequence ISP\"}";
                            let resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(), body
                            );
                            let _ = stream.write_all(resp.as_bytes());
                            break; // success served; stop accepting further connections
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No incoming connection yet; sleep briefly and re-check stop flag.
                        thread::sleep(Duration::from_millis(20));
                        continue;
                    }
                    Err(_other) => {
                        // On other errors, break to avoid tight loop in tests.
                        break;
                    }
                }
            }
        });

        // Give the thread a tiny moment to enter its accept loop to avoid
        // races where the client immediately attempts to connect before the
        // server is polling.
        thread::sleep(Duration::from_millis(20));
        SequenceServer { url, stop, handle }
    }

    fn url(&self) -> &str { &self.url }

    fn stop(self) {
        // Signal and then poke the listener by making a best-effort connection
        // (in case the thread is sitting in accept right before a WouldBlock path).
        self.stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(self.url.replace("http://", ""));
        let _ = self.handle.join();
    }
}

#[test]
fn ip_api_sequence_eventual_success_after_failures() {
    let failures = 3; // first 3 attempts fail with 500
    let server = SequenceServer::start(failures);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client build");

    // Provide retries = failures + 1 so final success is reachable.
    let isp = check_vpn::ip_api::get_isp_with_client_and_url(&client, server.url(), failures + 1)
        .expect("expected eventual success after retries");
    assert_eq!(isp, "Sequence ISP");
    server.stop();
}

#[test]
fn ip_api_sequence_insufficient_retries_errors() {
    let failures = 2; // server will fail twice then succeed
    let server = SequenceServer::start(failures);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client build");

    // Provide retries = failures (not enough: only failure attempts) -> should error
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, server.url(), failures);
    assert!(res.is_err(), "expected error when retries are insufficient");
    // Ensure we shut down the test server even though we didn't reach its success response.
    server.stop();
}
