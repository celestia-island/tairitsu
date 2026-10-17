//! End-to-end host-surface tests: the http import's implementation
//! against a real local HTTP server, plus validation caps and the
//! linker registration smoke.

use std::io::{Read, Write};
use std::net::TcpListener;

use tairitsu_transport_host::{
    add_to_linker, CredentialsMode, HttpHost, RedirectMode, RequestOptions, TransportHostState,
};

/// Minimal single-connection HTTP server: replies with the canned
/// status/headers/body after swallowing one request.
fn spawn_server(reply_status: &str, reply_headers: &str, reply_body: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind ephemeral");
    let addr = listener.local_addr().expect("local addr");
    let owned_status = reply_status.to_string();
    let owned_headers = reply_headers.to_string();
    let owned_body = reply_body.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).expect("read request").max(1);
        let _ = n;
        let response = format!("{owned_status}\r\n{owned_headers}\r\n\r\n{owned_body}");
        stream.write_all(response.as_bytes()).expect("write reply");
        let _ = stream.flush();
        // Hold the socket briefly so the client can read the reply.
        std::thread::sleep(std::time::Duration::from_millis(100));
    });
    format!("http://{addr}/probe")
}

fn opts(method: &str, headers: Vec<(String, String)>, body: Option<Vec<u8>>) -> RequestOptions {
    RequestOptions {
        method: method.to_string(),
        headers,
        body,
        credentials: Some(CredentialsMode::SameOrigin),
        redirect: Some(RedirectMode::Follow),
        timeout_ms: Some(5_000),
    }
}

#[test]
fn request_round_trips_status_headers_body() {
    let url = spawn_server(
        "HTTP/1.1 200 OK",
        "content-type: application/json\r\nx-probe: mirror",
        r#"{"ok":true}"#,
    );
    let mut host = TransportHostState::new();
    let resp = host
        .request(url, opts("GET", vec![], None))
        .expect("request succeeds");
    assert_eq!(resp.status, 200);
    assert_eq!(resp.status_text, "OK");
    let header = |name: &str| -> String {
        resp.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_default()
    };
    assert_eq!(header("content-type"), "application/json");
    assert_eq!(header("x-probe"), "mirror");
    assert_eq!(resp.body, br#"{"ok":true}"#.to_vec());
}

#[test]
fn request_reports_non_2xx_without_error() {
    let url = spawn_server("HTTP/1.1 404 Not Found", "content-length: 0", "");
    let mut host = TransportHostState::new();
    let resp = host
        .request(url, opts("GET", vec![], None))
        .expect("non-2xx still maps");
    assert_eq!(resp.status, 404);
    assert_eq!(resp.status_text, "Not Found");
}

#[test]
fn request_sends_method_headers_and_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let url = format!("http://{addr}/echo");
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = vec![0u8; 16384];
        let n = stream.read(&mut buf).expect("read");
        let request = String::from_utf8_lossy(&buf[..n]).into_owned();
        stream
            .write_all(b"HTTP/1.1 204 No Content\r\ncontent-length: 0\r\n\r\n")
            .expect("write");
        request
    });
    let mut host = TransportHostState::new();
    let resp = host
        .request(
            url,
            opts(
                "POST",
                vec![("x-mark".to_string(), "sent".to_string())],
                Some(b"payload-bytes".to_vec()),
            ),
        )
        .expect("request succeeds");
    assert_eq!(resp.status, 204);
    let seen = server.join().expect("server thread");
    assert!(
        seen.starts_with("POST /echo "),
        "method+path on the wire: {seen}"
    );
    assert!(seen.contains("x-mark: sent"), "header on the wire: {seen}");
    assert!(seen.contains("payload-bytes"), "body on the wire: {seen}");
}

#[test]
fn validation_caps_reject_oversized_input() {
    let mut many_headers = opts("GET", vec![], None);
    many_headers.headers = (0..100)
        .map(|i| (format!("h{i}"), "v".to_string()))
        .collect();
    let mut host = TransportHostState::new();
    let err = host
        .request("http://127.0.0.1:1/never".to_string(), many_headers)
        .expect_err("100 headers must be rejected before the network");
    assert!(err.contains("too many headers"), "{err}");

    let mut huge_body = opts("GET", vec![], None);
    huge_body.body = Some(vec![0u8; 9 * 1024 * 1024]);
    let err = host
        .request("http://127.0.0.1:1/never".to_string(), huge_body)
        .expect_err("9MiB body must be rejected before the network");
    assert!(err.contains("body too large"), "{err}");
}

#[test]
fn linker_registers_the_whole_transport_world() {
    let engine = wasmtime::Engine::default();
    let mut linker = wasmtime::component::Linker::<TransportHostState>::new(&engine);
    // All three interfaces (http, websocket, sse) register without
    // conflicting export names — an error here means the WIT contract
    // and the host impls drifted apart.
    add_to_linker(&mut linker).expect("all three interfaces register");
}
