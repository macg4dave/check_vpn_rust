use std::net::TcpListener;

#[test]
fn local_tcp_listener_is_detected() {
    // Bind to a free port on localhost
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind listener");
    let addr = listener.local_addr().expect("failed to get local addr");

    // The listener exists and will accept connections; is_online should return true
    let endpoints = [format!("127.0.0.1:{}", addr.port())];
    let ok = check_vpn::networking::is_online(&endpoints, 1).expect("is_online failed");
    assert!(ok, "expected local listener to be reachable");

    // Drop the listener when test ends
    drop(listener);
}

#[test]
fn unreachable_host_returns_false() {
    // Deterministic local simulation of an unreachable endpoint:
    // Use loopback with a high port that is extremely unlikely to have a
    // listener in typical CI/dev environments (65000). This avoids relying
    // on external networks and keeps the test deterministic.
    let endpoints = ["127.0.0.1:65000".to_string()];
    let ok = check_vpn::networking::is_online(&endpoints, 1).expect("is_online failed");
    assert!(!ok, "expected high-numbered loopback port to be unreachable");
}

#[test]
fn transient_listener_becomes_available() {
    // Reserve a port by binding to 0, then drop to allow reuse by the background thread.
    let reserver = std::net::TcpListener::bind("127.0.0.1:0").expect("failed to bind reserver");
    let port = reserver.local_addr().expect("local addr").port();
    drop(reserver);

    // Spawn a thread that will bind the port after a short delay to simulate a
    // service that comes up shortly after the check starts.
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
            // accept a single connection and then exit
            let _ = listener.accept();
        }
    });

    // Use the retries-enabled API; allow multiple attempts so the transient
    // listener has time to appear.
    let endpoints = ["127.0.0.1".to_string(); 1];
    let ok = check_vpn::networking::is_online_with_retries(&endpoints, 1, &[port], 8).expect("is_online failed");
    assert!(ok, "expected transient listener to be detected by retries");
}
