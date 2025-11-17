use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

/// Integration test: start the real metrics server thread and perform raw TCP
/// GET requests to exercise the accept/read/write loop and ensure responses
/// contain expected contents.
///
/// This test is marked `#[ignore]` because it starts a real TCP server and
/// therefore is intended to be run explicitly in CI or locally when needed.
#[test]
#[ignore]
fn metrics_server_serves_health_and_metrics() {
    // Reserve a free port by binding to 0, then drop to allow server to bind it.
    let reserver = TcpListener::bind("127.0.0.1:0").expect("bind reserver");
    let port = reserver.local_addr().expect("local addr").port();
    drop(reserver);

    let addr = format!("127.0.0.1:{}", port);

    let keep_running = Arc::new(AtomicBool::new(true));
    let handle = check_vpn::metrics::start_metrics_server(&addr, keep_running.clone())
        .expect("failed to start metrics server");

    // Wait until the server appears to be listening (try connecting a few times).
    let start = Instant::now();
    loop {
        if TcpListener::bind("127.0.0.1:0").is_ok() {
            // noop; binding succeeds but we need to test connect instead
        }
        match std::net::TcpStream::connect(&addr) {
            Ok(_) => break,
            Err(_) => {
                if start.elapsed() > Duration::from_millis(500) {
                    panic!("server did not start listening in time");
                }
                std::thread::sleep(Duration::from_millis(10));
            }
        }
    }

    // Helper to perform a GET and return the response string
    let do_get = |path: &str| -> String {
        let mut s = std::net::TcpStream::connect(&addr).expect("connect");
        let req = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, addr
        );
        s.write_all(req.as_bytes()).expect("write req");
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).expect("read resp");
        String::from_utf8_lossy(&buf).into_owned()
    };

    let health = do_get("/health");
    // Basic checks
    assert!(
        health.contains("200 OK"),
        "health status missing: {}",
        health
    );
    assert!(health.contains("ok"), "health body missing: {}", health);

    // Validate headers and content-length for health
    if let Some((raw_headers, body)) = health.split_once("\r\n\r\n") {
        let headers = raw_headers.lines().collect::<Vec<_>>();
        // Find Content-Length header
        let mut content_length: Option<usize> = None;
        for h in &headers {
            if let Some(v) = h.strip_prefix("Content-Length: ") {
                content_length = v.trim().parse::<usize>().ok();
            }
        }
        if let Some(len) = content_length {
            assert_eq!(len, body.len(), "health Content-Length mismatch");
        }
    }

    let metrics = do_get("/metrics");
    assert!(
        metrics.contains("200 OK"),
        "metrics status missing: {}",
        metrics
    );
    assert!(
        metrics.contains("check_vpn_up 1"),
        "metrics body missing: {}",
        metrics
    );

    // Validate headers and content-length for metrics and that content-type indicates metrics format
    if let Some((raw_headers, body)) = metrics.split_once("\r\n\r\n") {
        let headers = raw_headers.lines().collect::<Vec<_>>();
        let mut content_length: Option<usize> = None;
        let mut content_type: Option<String> = None;
        for h in &headers {
            if let Some(v) = h.strip_prefix("Content-Length: ") {
                content_length = v.trim().parse::<usize>().ok();
            }
            if let Some(v) = h.strip_prefix("Content-Type: ") {
                content_type = Some(v.trim().to_string());
            }
        }
        if let Some(len) = content_length {
            assert_eq!(len, body.len(), "metrics Content-Length mismatch");
        }
        if let Some(ct) = content_type {
            assert!(ct.contains("text/plain"), "unexpected Content-Type: {}", ct);
        }
    }

    // Concurrent clients: spawn several threads to fetch /metrics concurrently
    let mut handles = Vec::new();
    for _ in 0..5 {
        let addr_clone = addr.clone();
        handles.push(std::thread::spawn(move || {
            let mut s = std::net::TcpStream::connect(&addr_clone).expect("connect concurrent");
            let req = format!(
                "GET /metrics HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
                addr_clone
            );
            s.write_all(req.as_bytes()).expect("write req");
            let mut buf = Vec::new();
            s.read_to_end(&mut buf).expect("read resp");
            String::from_utf8_lossy(&buf).into_owned()
        }));
    }

    for h in handles {
        let resp = h.join().expect("thread join");
        assert!(resp.contains("check_vpn_up 1"));
    }

    // Shutdown server and join
    keep_running.store(false, Ordering::SeqCst);
    let _ = handle.join();
}
