use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::time::{Duration, Instant};

// Simple sequence server: first request -> 429 with Retry-After: 1, second -> 200 with JSON
#[test]
fn retry_after_respected_and_eventually_succeeds() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}{}", addr.ip(), addr.port(), "/json");

    // spawn thread to handle two connections
    let handle = thread::spawn(move || {
        // first connection: respond 429 with Retry-After: 1
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let resp = "HTTP/1.1 429 Too Many Requests\r\nRetry-After: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(resp.as_bytes());
        }
        // second connection: respond 200 with JSON
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            let body = r#"{"isp":"Retry ISP"}"#;
            let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body);
            let _ = stream.write_all(resp.as_bytes());
        }
    });

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(5)).build().unwrap();

    let start = Instant::now();
    let isp = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 2).expect("expected eventual success");
    let elapsed = start.elapsed();
    assert_eq!(isp, "Retry ISP");
    assert!(elapsed >= Duration::from_secs(1), "expected at least Retry-After delay");

    let _ = handle.join();
}
