//! End-to-end bridge tests: a real local tungstenite echo server for
//! the websocket poll bridge, and a raw text/event-stream server for
//! the sse bridge.

use std::io::Write;
use std::net::TcpListener;

use tairitsu_transport_host::{SseHost, TransportHostState, WebsocketHost, WsFrame};

fn wait_for<F: FnMut() -> Option<T>, T>(mut f: F, ms: u64) -> Option<T> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms);
    while std::time::Instant::now() < deadline {
        if let Some(v) = f() {
            return Some(v);
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    None
}

/// Tokio-tungstenite echo server: replies "echo:<text>" to every text.
fn spawn_ws_echo_server() -> String {
    use futures_util::{SinkExt, StreamExt};
    let (addr_tx, addr_rx) = std::sync::mpsc::channel::<std::net::SocketAddr>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind");
            addr_tx
                .send(listener.local_addr().expect("addr"))
                .expect("addr channel");
            let (stream, _) = listener.accept().await.expect("accept");
            let ws = tokio_tungstenite::accept_async(stream)
                .await
                .expect("accept ws");
            let (mut sink, mut stream) = ws.split();
            while let Some(Ok(msg)) = stream.next().await {
                if let tokio_tungstenite::tungstenite::Message::Text(t) = msg {
                    let reply = format!("echo:{}", t.as_str());
                    if sink
                        .send(tokio_tungstenite::tungstenite::Message::text(reply))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        });
    });
    let addr = addr_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("bind addr");
    format!("ws://127.0.0.1:{}/rpc", addr.port())
}

/// Raw SSE server writing the given payload after the stream headers.
fn spawn_sse_server(payload: &str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let payload = payload.to_string();
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        // Swallow the request head.
        let mut buf = [0u8; 4096];
        use std::io::Read;
        let _ = stream.read(&mut buf);
        let head = "HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\r\n";
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(payload.as_bytes());
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(400));
    });
    format!("http://127.0.0.1:{}/events", addr.port())
}

#[test]
fn websocket_bridge_round_trips_and_closes() {
    let url = spawn_ws_echo_server();
    let mut host = TransportHostState::new();
    let handle = host
        .connect(url, vec![])
        .expect("connect to the echo server");

    host.send(handle, WsFrame::Text("ping".into()))
        .expect("send");
    let echoed =
        wait_for(|| host.recv(handle).map(|f| format!("{f:?}")), 5_000).expect("echo arrived");
    assert!(echoed.contains("echo:ping"), "{echoed}");

    WebsocketHost::close(&mut host, handle, 1000, "done".into());
    let info = wait_for(|| host.close_info(handle), 5_000).expect("close info recorded");
    assert_eq!(info.code, 1000);
    assert_eq!(info.reason, "done");
    assert!(info.clean);

    // After close, recv drains to None and the peer side sees the close.
    assert!(host.recv(handle).is_none());
}

#[test]
fn websocket_bridge_records_server_initiated_close() {
    use futures_util::{SinkExt, StreamExt};
    let (addr_tx, addr_rx) = std::sync::mpsc::channel::<std::net::SocketAddr>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("rt");
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind");
            addr_tx
                .send(listener.local_addr().expect("addr"))
                .expect("ch");
            let (stream, _) = listener.accept().await.expect("accept");
            let ws = tokio_tungstenite::accept_async(stream)
                .await
                .expect("accept");
            let (mut sink, mut stream) = ws.split();
            // Wait for the client's greeting, then the SERVER closes with 1011.
            while let Some(Ok(msg)) = stream.next().await {
                if matches!(msg, tokio_tungstenite::tungstenite::Message::Text(_)) {
                    break;
                }
            }
            use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
            let frame = CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Error,
                reason: "server tired".into(),
            };
            let _ = sink
                .send(tokio_tungstenite::tungstenite::Message::Close(Some(frame)))
                .await;
        });
    });
    let addr = addr_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("addr");
    let mut host = TransportHostState::new();
    let handle = host
        .connect(format!("ws://127.0.0.1:{}/rpc", addr.port()), vec![])
        .expect("connect");
    host.send(handle, WsFrame::Text("hi".into())).expect("send");
    let info = wait_for(|| host.close_info(handle), 5_000).expect("server close recorded");
    assert_eq!(info.code, 1011);
    assert_eq!(info.reason, "server tired");
    assert!(info.clean);
}

#[test]
fn websocket_bridge_refuses_unreachable() {
    // Bind-then-drop to get a definitely-closed port.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);
    let mut host = TransportHostState::new();
    let err = host
        .connect(format!("ws://127.0.0.1:{port}/x"), vec![])
        .expect_err("connection refused must surface");
    assert!(!err.is_empty());
}

#[test]
fn sse_bridge_parses_events_and_ids() {
    let url = spawn_sse_server(
        "event: tick\ndata: one\nid: 7\n\ndata: plain\n\nevent: drop\ndata: x\n\n",
    );
    let mut host = TransportHostState::new();
    let handle = host.subscribe(url, vec![], vec![]).expect("subscribe");

    let first = wait_for(|| host.next(handle), 5_000).expect("first event");
    assert_eq!(first.event.as_deref(), Some("tick"));
    assert_eq!(first.data, "one");
    assert_eq!(first.id.as_deref(), Some("7"));

    let second = wait_for(|| host.next(handle), 5_000).expect("second event");
    assert_eq!(second.event, None);
    assert_eq!(second.data, "plain");

    let third = wait_for(|| host.next(handle), 5_000).expect("third event");
    assert_eq!(third.event.as_deref(), Some("drop"));

    // Stream ends → next reports None from now on.
    assert!(host.next(handle).is_none());
}

#[test]
fn sse_bridge_filters_named_events() {
    let url = spawn_sse_server("event: keep\ndata: k\n\nevent: skip\ndata: s\n\n");
    let mut host = TransportHostState::new();
    let handle = host
        .subscribe(url, vec![], vec!["keep".to_string()])
        .expect("subscribe");
    let event = wait_for(|| host.next(handle), 5_000).expect("kept event");
    assert_eq!(event.event.as_deref(), Some("keep"));
    // The `skip` event never lands; the stream ends shortly after.
    let extra = wait_for(|| host.next(handle), 1_500);
    assert!(extra.is_none(), "filtered event must not arrive: {extra:?}");
}

#[test]
fn sse_bridge_survives_long_silence() {
    // Regression for the review finding: the first bridge read through
    // reqwest's blocking facade, whose DEFAULT 30s timeout silently
    // killed quiet streams. The pump bridge must survive 35s of silence
    // and still deliver the late event.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        use std::io::Read;
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\ncontent-type: text/event-stream\r\n\r\ndata: first\n\n",
        );
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_secs(35));
        let _ = stream.write_all(b"data: late\n\n");
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(500));
    });
    let mut host = TransportHostState::new();
    let handle = host
        .subscribe(
            format!("http://127.0.0.1:{}/events", addr.port()),
            vec![],
            vec![],
        )
        .expect("subscribe");
    let first = wait_for(|| host.next(handle), 5_000).expect("first event");
    assert_eq!(first.data, "first");
    let late = wait_for(|| host.next(handle), 40_000)
        .expect("the late event must arrive after 35s of silence");
    assert_eq!(late.data, "late");
}

#[test]
fn sse_bridge_refuses_non_event_stream() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        use std::io::Read;
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\n\r\n{}");
    });
    let mut host = TransportHostState::new();
    let err = host
        .subscribe(
            format!("http://127.0.0.1:{}/x", addr.port()),
            vec![],
            vec![],
        )
        .expect_err("non-stream content type must refuse");
    assert!(err.contains("text/event-stream"), "{err}");
}
