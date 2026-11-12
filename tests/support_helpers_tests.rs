mod support;

use std::net::TcpStream;
use std::time::Duration;

#[test]
fn dropping_http_server_releases_listener() {
    let server = support::start_http_server("{}", 200);
    let address = server.addr.parse().expect("socket address");
    drop(server);
    assert!(TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_err());
}

#[test]
fn explicit_server_stop_releases_listener() {
    let server = support::start_http_server("{}", 200);
    let address = server.addr.parse().expect("socket address");
    server.stop();
    assert!(TcpStream::connect_timeout(&address, Duration::from_secs(1)).is_err());
}
