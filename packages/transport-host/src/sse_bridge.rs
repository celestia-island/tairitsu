//! Poll-style SSE bridge for the transport host.
//!
//! Mirrors the websocket bridge's architecture: `subscribe` performs the
//! HTTP GET synchronously (async client parked on the handshake), then a
//! pump thread owns a current-thread tokio runtime and `select!`s
//! between the response byte stream and a stop channel — no blocking
//! reads, so a host-state drop tears the reader down promptly (the
//! first version's blocking reader died at reqwest's default 30s
//! timeout; the review round caught it).
//!
//! `next` is a non-blocking try_recv; the event/data/id grammar follows
//! WHATWG server-sent events (comments and unknown fields ignored,
//! multi-line data joined with \n, sticky ids).

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;

use super::bridge_types::SseEventInternal;

/// A live SSE stream.
pub(super) struct SseSession {
    pub events: Receiver<SseEventInternal>,
    pub stop: tokio::sync::mpsc::UnboundedSender<()>,
}

#[derive(Default)]
pub(super) struct SseSessions {
    next: u64,
    map: HashMap<u64, SseSession>,
}

/// Components are untrusted: cap sessions like the other faces.
const MAX_SESSIONS: usize = 64;
/// Handshake wait — `subscribe` is synchronous from the component's view.
const SUBSCRIBE_TIMEOUT: Duration = Duration::from_secs(30);
/// Cap a single stream's delivered bytes (hostile-endpoint posture).
const MAX_DELIVERED_BYTES: usize = 32 * 1024 * 1024;

impl SseSessions {
    pub fn subscribe(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        events: Vec<String>,
    ) -> Result<u64, String> {
        if self.map.len() >= MAX_SESSIONS {
            return Err(format!(
                "too many sse sessions (cap {MAX_SESSIONS}, counted for the host state's lifetime); the runtime scopes a state per component instance"
            ));
        }
        if url.len() > 8192 {
            return Err("sse url too long".into());
        }
        for (name, value) in headers.iter().take(64) {
            if name.len() + value.len() > 8 * 1024 {
                return Err("sse header too large".into());
            }
        }

        let (event_tx, event_rx): (
            std::sync::mpsc::Sender<SseEventInternal>,
            Receiver<SseEventInternal>,
        ) = std::sync::mpsc::channel();
        let (stop_tx, mut stop_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let wanted = events;

        std::thread::Builder::new()
            .name("transport-host-sse".into())
            .spawn(move || {
                let rt = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(rt) => rt,
                    Err(e) => {
                        let _ = ready_tx.send(Err(format!("runtime build: {e}")));
                        return;
                    }
                };
                rt.block_on(async move {
                    // No request timeout: streams live as long as the
                    // server keeps them open; teardown rides the stop
                    // channel, not a timer.
                    let client = match reqwest::Client::builder().build() {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = ready_tx.send(Err(format!("client build: {e}")));
                            return;
                        }
                    };
                    let mut builder = client.get(&url);
                    for (name, value) in headers.iter().take(64) {
                        builder = builder.header(name.as_str(), value.as_str());
                    }
                    let response = match builder.send().await {
                        Ok(r) => r,
                        Err(e) => {
                            let _ =
                                ready_tx.send(Err(format!("sse subscribe to {url} failed: {e}")));
                            return;
                        }
                    };
                    let status = response.status();
                    if !status.is_success() {
                        let _ = ready_tx.send(Err(format!(
                            "sse subscribe to {url} returned HTTP {}",
                            status.as_u16()
                        )));
                        return;
                    }
                    let content_type = response
                        .headers()
                        .get(reqwest::header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("")
                        .to_string();
                    if !content_type.starts_with("text/event-stream") {
                        let _ = ready_tx.send(Err(format!(
                            "sse subscribe to {url} returned content-type {content_type:?}, not text/event-stream"
                        )));
                        return;
                    }
                    if ready_tx.send(Ok(())).is_err() {
                        return; // host gave up during the handshake
                    }

                    let mut stream = response.bytes_stream();
                    use futures_util::StreamExt;
                    let mut buffer: Vec<u8> = Vec::new();
                    let mut event_name: Option<String> = None;
                    let mut data_lines: Vec<String> = Vec::new();
                    let mut last_id: Option<String> = None;
                    let mut delivered: usize = 0;

                    loop {
                        let chunk = tokio::select! {
                            biased;
                            _ = stop_rx.recv() => return, // host closed the session
                            maybe = stream.next() => match maybe {
                                Some(Ok(bytes)) => bytes,
                                Some(Err(_)) | None => return, // stream ended
                            },
                        };
                        if delivered + chunk.len() > MAX_DELIVERED_BYTES {
                            return; // hostile stream exceeded the cap
                        }
                        buffer.extend_from_slice(&chunk);
                        // Parse every complete line currently buffered.
                        while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                            let line: Vec<u8> = buffer.drain(..=pos).collect();
                            let line = String::from_utf8_lossy(&line[..line.len() - 1])
                                .trim_end_matches('\r')
                                .to_string();
                            if line.is_empty() {
                                if !data_lines.is_empty() {
                                    // Filter semantics: an empty subscribe
                                    // list takes the whole stream (named
                                    // AND unnamed); a non-empty list takes
                                    // only the named events it lists.
                                    let deliver = match &event_name {
                                        Some(n) => wanted.is_empty() || wanted.iter().any(|w| w == n),
                                        None => wanted.is_empty(),
                                    };
                                    if deliver {
                                        let event = SseEventInternal {
                                            event: event_name.take(),
                                            data: data_lines.join("\n"),
                                            id: last_id.clone(),
                                        };
                                        delivered += event.data.len();
                                        if event_tx.send(event).is_err() {
                                            return; // host dropped the session
                                        }
                                    }
                                    data_lines.clear();
                                } else {
                                    event_name = None;
                                }
                                continue;
                            }
                            if let Some(rest) = line.strip_prefix("event:") {
                                event_name =
                                    Some(rest.strip_prefix(' ').unwrap_or(rest).to_string());
                            } else if let Some(rest) = line.strip_prefix("data:") {
                                data_lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
                            } else if let Some(rest) = line.strip_prefix("id:") {
                                last_id = Some(rest.trim().to_string());
                            }
                            // comments (`:` prefix) and unknown fields are ignored
                        }
                        // Cap the pending buffer for pathological no-newline streams.
                        if buffer.len() > 1024 * 1024 {
                            return;
                        }
                    }
                })
            })
            .map_err(|e| format!("spawn sse pump thread: {e}"))?;

        match ready_rx.recv_timeout(SUBSCRIBE_TIMEOUT) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("sse subscribe timed out".into()),
        }

        self.next += 1;
        let id = self.next;
        self.map.insert(
            id,
            SseSession {
                events: event_rx,
                stop: stop_tx,
            },
        );
        Ok(id)
    }

    pub fn next(&mut self, handle: u64) -> Option<SseEventInternal> {
        let session = self.map.get(&handle)?;
        match session.events.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn close(&mut self, handle: u64) {
        if let Some(session) = self.map.get(&handle) {
            let _ = session.stop.send(());
        }
    }
}
