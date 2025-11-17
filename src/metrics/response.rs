/// Small helpers to build HTTP responses used by the metrics server.
/// Returning Vec<u8> keeps the server code simple and avoids repeated
/// formatting at the accept-path.
pub fn build_health_response() -> Vec<u8> {
    let body = "ok";
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    resp.into_bytes()
}

pub fn build_metrics_response() -> Vec<u8> {
    let body =
        "# HELP check_vpn_up 1 if the service is up\n# TYPE check_vpn_up gauge\ncheck_vpn_up 1\n";
    let resp = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    resp.into_bytes()
}

pub fn build_not_found_response() -> Vec<u8> {
    let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    resp.as_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_response_contains_ok() {
        let b = build_health_response();
        let s = String::from_utf8(b).unwrap();
        assert!(s.contains("200 OK"));
        assert!(s.ends_with("ok"));
    }

    #[test]
    fn metrics_response_contains_metric() {
        let b = build_metrics_response();
        let s = String::from_utf8(b).unwrap();
        assert!(s.contains("check_vpn_up 1"));
        assert!(s.contains("200 OK"));
    }

    #[test]
    fn not_found_response_is_404() {
        let b = build_not_found_response();
        let s = String::from_utf8(b).unwrap();
        assert!(s.contains("404 Not Found"));
    }
}
