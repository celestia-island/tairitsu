//! Tairitsu MCP Browser Plugin — standalone subprocess.
//!
//! Communicates with the host (tairitsu) over a Unix domain socket
//! using the protocol defined in `tairitsu-shared`.
//!
//! Usage:
//!
//!     tairitsu-plugin-debug-browser --socket /tmp/tairitsu-plugins/tairitsu-plugin-debug-browser.sock

use std::path::PathBuf;

use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, ListenerOptions, ToFsName,
};
use tairitsu_shared::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PLUGIN_NAME: &str = "debug-browser";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_DESCRIPTION: &str = "Headless Chromium CDP automation via debug HTTP API";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tairitsu_mcp_plugin_browser=info".parse()?),
        )
        .init();

    let socket_path = parse_args()?;

    // Remove stale socket if present
    let _ = std::fs::remove_file(&socket_path);

    let listener = ListenerOptions::new()
        .name(socket_path.clone().to_fs_name::<GenericFilePath>()?)
        .create_tokio()
        .map_err(|e| anyhow::anyhow!("bind socket {}: {}", socket_path.display(), e))?;

    tracing::info!("[{}] Listening on {}", PLUGIN_NAME, socket_path.display());

    // Accept one connection from host
    let stream: Stream = listener
        .accept()
        .await
        .map_err(|e| anyhow::anyhow!("accept connection: {}", e))?;

    tracing::info!("[{}] Host connected", PLUGIN_NAME);

    run_event_loop(stream).await?;

    Ok(())
}

/// Parse --socket <path> from command line arguments.
fn parse_args() -> anyhow::Result<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--socket" {
        anyhow::bail!(
            "Usage: {} --socket <socket-path>",
            args.get(0).map(|s| s.as_str()).unwrap_or("plugin")
        );
    }
    Ok(PathBuf::from(&args[2]))
}

/// Main event loop: handshake → request/response cycle.
async fn run_event_loop(stream: Stream) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    // Send handshake
    let handshake = Message::Handshake(Handshake {
        protocol_version: PROTOCOL_VERSION,
        name: PLUGIN_NAME.to_string(),
        version: PLUGIN_VERSION.to_string(),
        capabilities: vec![caps::DEBUG_BROWSER.to_string()],
        description: Some(PLUGIN_DESCRIPTION.to_string()),
    });

    send_message(&mut stream, &handshake).await?;
    tracing::info!("[{}] Handshake sent", PLUGIN_NAME);

    // Wait for ack (optional — plugin can proceed regardless)
    let mut line = String::new();
    if stream.read_line(&mut line).await? > 0 {
        if let Ok(msg) = serde_json::from_str::<Message>(line.trim()) {
            match msg {
                Message::HandshakeAck(ack) => {
                    if !ack.accepted {
                        tracing::error!(
                            "[{}] Handshake rejected: {:?}",
                            PLUGIN_NAME,
                            ack.reason
                        );
                        return Ok(());
                    }
                    tracing::info!("[{}] Handshake accepted", PLUGIN_NAME);
                }
                other => {
                    tracing::warn!(
                        "[{}] Unexpected message after handshake: {:?}",
                        PLUGIN_NAME,
                        other
                    );
                }
            }
        }
    }

    // Request/response loop
    loop {
        line.clear();
        let bytes_read = stream.read_line(&mut line).await?;
        if bytes_read == 0 {
            tracing::info!("[{}] Host disconnected", PLUGIN_NAME);
            break;
        }

        let msg: Message = match serde_json::from_str(line.trim()) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "[{}] Invalid message: {} ({})",
                    PLUGIN_NAME,
                    e,
                    line.trim()
                );
                continue;
            }
        };

        match msg {
            Message::Request(req) => {
                let resp = handle_request(&req).await;
                send_message(&mut stream, &resp).await?;
            }
            _ => {
                tracing::warn!(
                    "[{}] Unexpected message type in loop: {:?}",
                    PLUGIN_NAME,
                    msg
                );
            }
        }
    }

    Ok(())
}

/// Handle an incoming request and produce a response.
async fn handle_request(req: &Request) -> Message {
    match req.method.as_str() {
        "ping" => Message::Response(Response {
            id: req.id,
            result: Some(serde_json::json!({ "pong": true })),
        }),
        "browser.navigate" => {
            let url = req
                .params
                .as_ref()
                .and_then(|p| p.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            tracing::info!("[{}] navigate: {}", PLUGIN_NAME, url);
            // TODO: implement actual browser navigation via chromiumoxide
            Message::Response(Response {
                id: req.id,
                result: Some(serde_json::json!({
                    "url": url,
                    "title": String::new(),
                })),
            })
        }
        "browser.screenshot" => {
            tracing::info!("[{}] screenshot requested", PLUGIN_NAME);
            // TODO: implement actual screenshot capture
            Message::Response(Response {
                id: req.id,
                result: Some(serde_json::json!({
                    "data": "",
                    "format": "png",
                })),
            })
        }
        "browser.click"
        | "browser.type"
        | "browser.press"
        | "browser.snapshot"
        | "browser.evaluate" => {
            tracing::info!("[{}] {}", PLUGIN_NAME, req.method);
            // TODO: implement each method
            Message::Response(Response {
                id: req.id,
                result: Some(serde_json::json!(null)),
            })
        }
        other => {
            tracing::warn!("[{}] Unknown method: {}", PLUGIN_NAME, other);
            Message::Error(ErrorResponse {
                id: req.id,
                error: ErrorBody {
                    code: -1,
                    message: format!("Unknown method: {}", other),
                },
            })
        }
    }
}

/// Serialize and send a message over the socket (newline-delimited JSON).
async fn send_message(
    stream: &mut BufReader<Stream>,
    msg: &Message,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    stream.get_mut().write_all(json.as_bytes()).await?;
    stream.get_mut().write_all(b"\n").await?;
    stream.get_mut().flush().await?;
    Ok(())
}
