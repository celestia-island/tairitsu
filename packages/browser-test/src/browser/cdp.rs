//! Minimal raw-CDP client for the browser-test harness.
//!
//! browser-test previously depended on `chromiumoxide`, which is unmaintained
//! (its frozen CDP schema can't parse Chrome >=147 events, so the harness was
//! broken on modern Chrome). This is a tiny, self-contained client that does
//! exactly what the harness needs — launch chrome, open its page devtools
//! websocket, and `Runtime.evaluate` the conformance scripts. Commands are
//! correlated by id and unknown events are ignored, so it's skew-proof by the
//! same construction as the packager's debug engine.

use anyhow::{Context, Result};
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{
    process::{Child, Command},
    sync::{mpsc, oneshot, Mutex},
};

use tokio_tungstenite::tungstenite::Message;

const CMD_TIMEOUT: Duration = Duration::from_secs(30);

pub struct CdpClient {
    inner: Arc<CdpInner>,
    /// Owning handle so chrome is reaped when the client drops.
    _child: Child,
}

struct CdpInner {
    outbox: mpsc::UnboundedSender<String>,
    pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
    next_id: AtomicU64,
}

impl CdpClient {
    /// Launch chrome at `exe`, open `initial_url`, and connect its page-level
    /// devtools websocket.
    pub async fn launch(exe: &Path, headless: bool, initial_url: &str) -> Result<Self> {
        let port = std::net::TcpListener::bind(("127.0.0.1", 0))
            .ok()
            .and_then(|l| l.local_addr().ok())
            .map(|a| a.port())
            .context("no free port for devtools")?;

        let mut args: Vec<String> = Vec::new();
        if headless {
            args.push("--headless=new".into());
        }
        args.extend([
            "--no-sandbox".into(),
            "--disable-dev-shm-usage".into(),
            "--disable-gpu".into(),
            "--disable-extensions".into(),
            "--no-first-run".into(),
            format!("--remote-debugging-port={port}"),
            "--window-size=1280,720".into(),
            initial_url.to_string(),
        ]);

        let child = Command::new(exe)
            .args(&args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("failed to launch chrome at {}", exe.display()))?;

        let ws_url = wait_devtools(port).await?;
        let (ws, _resp) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .context("devtools ws connect failed")?;
        let (mut sink, mut stream) = ws.split();
        let (outbox, mut inbox) = mpsc::unbounded_channel::<String>();
        let inner = Arc::new(CdpInner {
            outbox,
            pending: Mutex::new(HashMap::new()),
            next_id: AtomicU64::new(0),
        });

        // outbound JSON → ws sink
        tokio::spawn(async move {
            while let Some(raw) = inbox.recv().await {
                if sink.send(Message::Text(raw.into())).await.is_err() {
                    break;
                }
            }
            let _ = sink.send(Message::Close(None)).await;
        });
        // inbound: resolve responses by id, ignore events
        {
            let inner = inner.clone();
            tokio::spawn(async move {
                while let Some(msg) = stream.next().await {
                    let text = match msg {
                        Ok(Message::Text(t)) => t.to_string(),
                        Ok(Message::Ping(_)) => continue,
                        Ok(_) => continue,
                        Err(_) => break,
                    };
                    let v: Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                        if let Some(tx) = inner.pending.lock().await.remove(&id) {
                            let result = if let Some(err) = v.get("error") {
                                Err(err
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("cdp error")
                                    .to_string())
                            } else {
                                Ok(v.get("result").cloned().unwrap_or(Value::Null))
                            };
                            let _ = tx.send(result);
                        }
                    }
                }
            });
        }

        let client = CdpClient {
            inner,
            _child: child,
        };
        let _ = client.command("Page.enable", json!({})).await;
        let _ = client.command("Runtime.enable", json!({})).await;
        Ok(client)
    }

    async fn command(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst) + 1;
        let payload = json!({ "id": id, "method": method, "params": params });
        let (tx, rx) = oneshot::channel();
        self.inner.pending.lock().await.insert(id, tx);
        let raw = serde_json::to_string(&payload)?;
        self.inner
            .outbox
            .send(raw)
            .map_err(|_| anyhow::anyhow!("cdp writer closed"))?;
        match tokio::time::timeout(CMD_TIMEOUT, rx).await {
            Ok(Ok(v)) => v.map_err(anyhow::Error::msg),
            Ok(Err(_)) => anyhow::bail!("cdp response channel closed"),
            Err(_) => {
                self.inner.pending.lock().await.remove(&id);
                anyhow::bail!("cdp command '{method}' timed out")
            }
        }
    }

    /// `Runtime.evaluate` with `returnByValue`; returns the JS value (or the
    /// exception message on throw).
    pub async fn evaluate(&self, expression: &str) -> Result<Value> {
        let resp = self
            .command(
                "Runtime.evaluate",
                json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": true,
                }),
            )
            .await?;
        if let Some(exc) = resp.get("exceptionDetails") {
            let msg = exc
                .get("exception")
                .and_then(|e| e.get("description"))
                .and_then(|d| d.as_str())
                .or_else(|| exc.get("text").and_then(|t| t.as_str()))
                .unwrap_or("runtime exception");
            anyhow::bail!(msg.to_string());
        }
        Ok(resp
            .get("result")
            .and_then(|r| r.get("value"))
            .cloned()
            .unwrap_or(Value::Null))
    }
}

/// Poll `/json/list` until chrome's devtools is up; return the first page
/// target's websocket URL (Page.*/Runtime.* only work on a page target).
async fn wait_devtools(port: u16) -> Result<String> {
    let list_url = format!("http://127.0.0.1:{port}/json/list");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()?;
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    loop {
        if std::time::Instant::now() > deadline {
            anyhow::bail!("devtools never came up on :{port}");
        }
        if let Ok(resp) = client.get(&list_url).send().await {
            if resp.status().is_success() {
                if let Ok(Value::Array(targets)) = resp.json::<Value>().await {
                    for t in &targets {
                        if t.get("type").and_then(|v| v.as_str()) == Some("page") {
                            if let Some(ws) = t
                                .get("webSocketDebuggerUrl")
                                .and_then(|w| w.as_str())
                                .map(String::from)
                            {
                                return Ok(ws);
                            }
                        }
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}
