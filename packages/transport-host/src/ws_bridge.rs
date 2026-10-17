//! Poll-style websocket bridge for the transport host.
//!
//! The WIT surface is synchronous (`recv: func(handle) -> option<frame>`);
//! tokio-tungstenite is async. Each session therefore owns an OS thread
//! running a current-thread tokio runtime: the thread pumps socket ↔
//! unbounded channels with `select!`, and the host methods just move
//! items across — `recv` is a non-blocking try_recv (None = nothing
//! yet), exactly the poll bridge the contract describes.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use super::bridge_types::{WsCloseInfoInternal, WsFrameInternal};

/// Host→pump commands.
pub(super) enum PumpCmd {
    Send(WsFrameInternal),
    Close(u16, String),
}

/// A live (or closed) websocket session.
pub(super) struct WsSession {
    pub frames: UnboundedReceiver<WsFrameInternal>,
    pub cmds: UnboundedSender<PumpCmd>,
    pub close_info: Arc<Mutex<Option<WsCloseInfoInternal>>>,
}

/// Session bookkeeping owned by the host state.
#[derive(Default)]
pub(super) struct WsSessions {
    next: u64,
    map: HashMap<u64, WsSession>,
}

/// Components are untrusted: cap concurrent sessions like everything else.
const MAX_SESSIONS: usize = 64;
/// Handshake wait — `connect` is synchronous from the component's view.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
/// Cap a single text frame's size (tungstenite config).
const MAX_FRAME_BYTES: usize = 4 * 1024 * 1024;

impl WsSessions {
    /// Connect (blocking up to CONNECT_TIMEOUT), then spawn the pump.
    pub fn connect(&mut self, url: String, protocols: Vec<String>) -> Result<u64, String> {
        if self.map.len() >= MAX_SESSIONS {
            return Err(format!(
                "too many websocket sessions (cap {MAX_SESSIONS}); close some first"
            ));
        }
        if url.len() > 8192 {
            return Err("websocket url too long".into());
        }

        let (frame_tx, frame_rx) = unbounded_channel::<WsFrameInternal>();
        let (cmd_tx, mut cmd_rx) = unbounded_channel::<PumpCmd>();
        let close_info = Arc::new(Mutex::new(None::<WsCloseInfoInternal>));
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let close_for_thread = Arc::clone(&close_info);

        std::thread::Builder::new()
            .name("transport-host-ws".into())
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
                    let mut request = match url.as_str().into_client_request() {
                        Ok(r) => r,
                        Err(e) => {
                            let _ = ready_tx.send(Err(format!("invalid url: {e}")));
                            return;
                        }
                    };
                    if !protocols.is_empty() && protocols.len() <= 16 {
                        let list = protocols.join(", ");
                        if let Ok(value) = list.parse() {
                            request
                                .headers_mut()
                                .insert("Sec-WebSocket-Protocol", value);
                        }
                    }
                    let config = WebSocketConfig::default().max_frame_size(Some(MAX_FRAME_BYTES));
                    let (ws, _resp) =
                        match tokio_tungstenite::connect_async_with_config(request, Some(config), false)
                            .await
                        {
                            Ok(pair) => pair,
                            Err(e) => {
                                let _ = ready_tx.send(Err(format!("connect: {e}")));
                                return;
                            }
                        };
                    if ready_tx.send(Ok(())).is_err() {
                        return; // host gave up during the handshake
                    }
                    let (mut sink, mut stream) = ws.split();
                    use futures_util::{SinkExt, StreamExt};
                    loop {
                        tokio::select! {
                            biased;
                            maybe_cmd = cmd_rx.recv() => match maybe_cmd {
                                Some(PumpCmd::Send(frame)) => {
                                    let msg = match frame {
                                        WsFrameInternal::Text(t) => Message::text(t),
                                        WsFrameInternal::Binary(b) => Message::binary(b),
                                    };
                                    if sink.send(msg).await.is_err() {
                                        return;
                                    }
                                }
                                Some(PumpCmd::Close(code, reason)) => {
                                    let frame = CloseFrame {
                                        code: code.into(),
                                        reason: reason.clone().into(),
                                    };
                                    let _ = sink.send(Message::Close(Some(frame))).await;
                                    *close_for_thread.lock().expect("close lock") =
                                        Some(WsCloseInfoInternal { code, reason, clean: true });
                                    return;
                                }
                                None => return,
                            },
                            maybe_msg = stream.next() => match maybe_msg {
                                Some(Ok(Message::Text(t))) => {
                                    if frame_tx.send(WsFrameInternal::Text(t.as_str().to_string())).is_err() {
                                        return; // host dropped the session
                                    }
                                }
                                Some(Ok(Message::Binary(b))) => {
                                    if frame_tx.send(WsFrameInternal::Binary(b.into())).is_err() {
                                        return;
                                    }
                                }
                                Some(Ok(Message::Close(frame))) => {
                                    let info = frame
                                        .map(|cf| WsCloseInfoInternal {
                                            code: u16::from(cf.code),
                                            reason: cf.reason.as_str().to_string(),
                                            clean: true,
                                        })
                                        .unwrap_or(WsCloseInfoInternal {
                                            code: 1005,
                                            reason: String::new(),
                                            clean: false,
                                        });
                                    *close_for_thread.lock().expect("close lock") = Some(info);
                                    return;
                                }
                                Some(Ok(_)) => { /* ping/pong are handled in-library */ }
                                Some(Err(_)) | None => {
                                    *close_for_thread.lock().expect("close lock") =
                                        Some(WsCloseInfoInternal {
                                            code: 1006,
                                            reason: String::new(),
                                            clean: false,
                                        });
                                    return;
                                }
                            },
                        }
                    }
                })
            })
            .map_err(|e| format!("spawn pump thread: {e}"))?;

        match ready_rx.recv_timeout(CONNECT_TIMEOUT) {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err("websocket handshake timed out".into()),
        }

        self.next += 1;
        let id = self.next;
        self.map.insert(
            id,
            WsSession {
                frames: frame_rx,
                cmds: cmd_tx,
                close_info,
            },
        );
        Ok(id)
    }

    pub fn send(&mut self, handle: u64, frame: WsFrameInternal) -> Result<(), String> {
        let session = self
            .map
            .get(&handle)
            .ok_or_else(|| format!("unknown websocket handle {handle}"))?;
        session
            .cmds
            .send(PumpCmd::Send(frame))
            .map_err(|_| "websocket session is gone".to_string())
    }

    pub fn recv(&mut self, handle: u64) -> Option<WsFrameInternal> {
        let session = self.map.get_mut(&handle)?;
        session.frames.try_recv().ok()
    }

    pub fn close_info(&mut self, handle: u64) -> Option<WsCloseInfoInternal> {
        let session = self.map.get(&handle)?;
        session.close_info.lock().expect("close lock").clone()
    }

    pub fn close(&mut self, handle: u64, code: u16, reason: String) {
        if let Some(session) = self.map.get(&handle) {
            // Best effort: a closed channel means the pump already exited.
            let _ = session.cmds.send(PumpCmd::Close(code, reason));
        }
    }
}
