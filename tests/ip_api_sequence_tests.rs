use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::thread;
use std::time::Duration;

// Spin up a minimal HTTP server that returns 500 for the first N requests, then 200 JSON.
fn start_sequence_server(failures: usize) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}{}", addr.ip(), addr.port(), "/json");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let handle = thread::spawn(move || {
        // Accept connections until we have served the success response.
        for stream_res in listener.incoming() {
            if let Ok(mut stream) = stream_res {
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
        }
    });

    (url, handle)
}

#[test]
fn ip_api_sequence_eventual_success_after_failures() {
    let failures = 3; // first 3 attempts fail with 500
    let (url, handle) = start_sequence_server(failures);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client build");

    // Provide retries = failures + 1 so final success is reachable.
    let isp = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, failures + 1)
        .expect("expected eventual success after retries");
    assert_eq!(isp, "Sequence ISP");

    let _ = handle.join();
}

#[test]
fn ip_api_sequence_insufficient_retries_errors() {
    let failures = 2; // server will fail twice then succeed
    let (url, handle) = start_sequence_server(failures);

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client build");

    // Provide retries = failures (not enough: only failure attempts) -> should error
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, failures);
    assert!(res.is_err(), "expected error when retries are insufficient");

    let _ = handle.join();
}
