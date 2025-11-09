use std::net::TcpListener;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

// Server that writes a truncated JSON body and closes connection immediately
#[test]
fn truncated_json_returns_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}{}", addr.ip(), addr.port(), "/json");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // write headers and a partial JSON body, then close
            let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 100\r\nConnection: close\r\n\r\n{\"isp\":\"Partial\"}";
            let _ = stream.write_all(resp.as_bytes());
            // close immediately
        }
    });

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(2)).build().unwrap();
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    let _ = handle.join();
    assert!(res.is_err(), "expected parse error for truncated JSON");
}

// Server that sends body very slowly in small chunks exceeding client timeout
#[test]
fn slow_chunked_response_times_out() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}{}", addr.ip(), addr.port(), "/json");

    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf);
            // write headers
            let header = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(header.as_bytes());
            // write body slowly
            let body = r#"{"isp":"Slow ISP"}"#;
            for b in body.as_bytes().chunks(1) {
                let _ = stream.write_all(b);
                thread::sleep(Duration::from_millis(200));
            }
        }
    });

    let client = reqwest::blocking::Client::builder().timeout(Duration::from_secs(1)).build().unwrap();
    let res = check_vpn::ip_api::get_isp_with_client_and_url(&client, &url, 1);
    let _ = handle.join();
    assert!(res.is_err(), "expected timeout for very slow response");
}
