use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{mpsc, oneshot, RwLock};

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{delete, get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
};

const DEBUG_API_VERSION: &str = "0.1.0";
const DEFAULT_VIEWPORT_W: u32 = 1280;
const DEFAULT_VIEWPORT_H: u32 = 720;
const OP_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }
    fn err(msg: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    api_version: String,
    uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InfoResponse {
    version: String,
    api_version: String,
    dev_port: u16,
    debug_port: u16,
    dist_dir: String,
    package_name: String,
    pid: u32,
    started_at_iso: String,
    uptime_secs: u64,
    browser_connected: bool,
    browser_engine: String,
    viewport: [u32; 2],
}

#[derive(Debug, Clone, Deserialize)]
struct NavigateRequest {
    url: String,
    wait_for: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigateResponse {
    url: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ScreenshotParams {
    selector: Option<String>,
    full_page: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenshotResponse {
    data: String,
    mime_type: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct ClickRequest {
    selector: String,
}

#[derive(Debug, Clone, Deserialize)]
struct TypeRequest {
    selector: String,
    text: String,
    clear_first: Option<bool>,
    submit: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvaluateRequest {
    expression: String,
    await_promise: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvaluateResponse {
    result: serde_json::Value,
    r#type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsoleEntry {
    level: String,
    text: String,
    timestamp: String,
    source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsoleResponse {
    entries: Vec<ConsoleEntry>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct DomQueryParams {
    selector: String,
    attribute: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomNodeResponse {
    tag: Option<String>,
    text: Option<String>,
    html: Option<String>,
    attributes: Option<serde_json::Map<String, serde_json::Value>>,
    visible: Option<bool>,
    count: usize,
    rect: Option<RectResponse>,
    computed: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RectResponse {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    children_visible: Option<usize>,
    overflowing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadyResponse {
    ready: bool,
    wasm_loaded: bool,
    hydrated: bool,
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ComputedStyleParams {
    selector: String,
    properties: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComputedStyleResponse {
    selector: String,
    properties: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct PressRequest {
    key: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ScrollRequest {
    selector: Option<String>,
    x: Option<f64>,
    y: Option<f64>,
    direction: Option<String>,
    amount: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ResizeRequest {
    width: Option<u32>,
    height: Option<u32>,
    preset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewportResponse {
    width: u32,
    height: u32,
    device_pixel_ratio: f64,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ConsoleQueryParams {
    level: Option<String>,
    source: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEntry {
    message: String,
    stack: Option<String>,
    r#type: String,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorsResponse {
    errors: Vec<ErrorEntry>,
    unhandled_rejections: Vec<ErrorEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct DragRequest {
    from_selector: String,
    to_selector: String,
    steps: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct A11yQueryParams {
    selector: Option<String>,
    depth: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct A11yNode {
    name: Option<String>,
    role: Option<String>,
    description: Option<String>,
    states: Vec<String>,
    tag: Option<String>,
    children: Vec<A11yNode>,
}

#[derive(Debug, Clone, Deserialize)]
struct BatchRequest {
    operations: Vec<BatchOperation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum BatchOperation {
    #[serde(rename = "navigate")]
    Navigate {
        url: String,
        wait_for: Option<String>,
    },
    #[serde(rename = "screenshot")]
    Screenshot {
        selector: Option<String>,
        full_page: Option<bool>,
        name: Option<String>,
    },
    #[serde(rename = "click")]
    Click { selector: String },
    #[serde(rename = "evaluate")]
    Evaluate { expression: String },
    #[serde(rename = "wait")]
    Wait { ms: u64 },
    #[serde(rename = "scroll")]
    Scroll {
        selector: Option<String>,
        direction: Option<String>,
        amount: Option<f64>,
    },
    #[serde(rename = "resize")]
    Resize {
        width: Option<u32>,
        height: Option<u32>,
        preset: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BatchResult {
    name: String,
    op_type: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkResource {
    name: String,
    r#type: String,
    duration: f64,
    size: f64,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkResponse {
    resources: Vec<NetworkResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PerformanceMetrics {
    dom_content_loaded_ms: Option<f64>,
    dom_complete_ms: Option<f64>,
    load_event_ms: Option<f64>,
    fcp_ms: Option<f64>,
    lcp_ms: Option<f64>,
    cls: Option<f64>,
    dom_nodes: u32,
    js_heap_used_mb: Option<f64>,
    wasm_loaded: bool,
    hydrated: bool,
    timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSocketInfo {
    active_count: u32,
    connections: Vec<WebSocketConn>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSocketConn {
    url: String,
    state: String,
    created_at_ms: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
struct SourceMapRequest {
    stack: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceMapResponse {
    frames: Vec<StackFrame>,
    raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StackFrame {
    file: String,
    line: Option<u32>,
    col: Option<u32>,
    func: Option<String>,
    raw: String,
}

// ── Browser command channel ───────────────────────────────────────────────

enum BrowserCommand {
    Navigate {
        url: String,
        wait_for: Option<String>,
        resp: oneshot::Sender<Result<NavigateResponse, String>>,
    },
    Screenshot {
        selector: Option<String>,
        full_page: bool,
        resp: oneshot::Sender<Result<ScreenshotResponse, String>>,
    },
    Click {
        selector: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    TypeText {
        selector: String,
        text: String,
        clear_first: bool,
        submit: bool,
        resp: oneshot::Sender<Result<(), String>>,
    },
    Evaluate {
        expression: String,
        await_promise: bool,
        resp: oneshot::Sender<Result<EvaluateResponse, String>>,
    },
    DomQuery {
        selector: String,
        attribute: Option<String>,
        computed: Option<Vec<String>>,
        resp: oneshot::Sender<Result<DomNodeResponse, String>>,
    },
    IsReady {
        resp: oneshot::Sender<Result<ReadyResponse, String>>,
    },
    Press {
        key: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    Scroll {
        selector: Option<String>,
        x: f64,
        y: f64,
        resp: oneshot::Sender<Result<(), String>>,
    },
    Resize {
        width: u32,
        height: u32,
        resp: oneshot::Sender<Result<(), String>>,
    },
    Viewport {
        resp: oneshot::Sender<Result<ViewportResponse, String>>,
    },
    Drag {
        from_selector: String,
        to_selector: String,
        steps: u32,
        resp: oneshot::Sender<Result<(), String>>,
    },
    A11y {
        selector: Option<String>,
        depth: u32,
        resp: oneshot::Sender<Result<Vec<A11yNode>, String>>,
    },
    Network {
        resp: oneshot::Sender<Result<NetworkResponse, String>>,
    },
    Performance {
        resp: oneshot::Sender<Result<PerformanceMetrics, String>>,
    },
    WebSocket {
        resp: oneshot::Sender<Result<WebSocketInfo, String>>,
    },
}

struct BrowserHandle {
    tx: mpsc::Sender<BrowserCommand>,
    connected: Arc<RwLock<bool>>,
}

impl BrowserHandle {
    async fn send(&self, cmd: BrowserCommand) -> Result<(), String> {
        self.tx.send(cmd).await.map_err(|e| e.to_string())
    }
    async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }
}

// ── Chromium-based Browser Engine (minimal raw-CDP client) ──────────────────
//
// A small, dependency-light CDP client: launch headless chromium, open its
// devtools websocket, and speak only the handful of CDP domains the debug API
// needs (Page / Runtime / Emulation). Outbound commands carry an `id`; inbound
// messages are dispatched by `id` and **everything else (events) is ignored**.
// Unlike a codegen'd CDP schema — which hard-breaks when Chrome ships a new
// event variant (the failure that killed chromiumoxide on Chrome ≥147) — this
// stays compatible across Chrome versions by construction: unknown events are
// dropped, never deserialized into a closed enum.

#[cfg(feature = "debug-browser")]
mod engine {
    use futures::{SinkExt, StreamExt};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::process::{Child, Command};
    use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
    use tokio_tungstenite::tungstenite::Message;

    use super::*;

    const DEVTOOLS_POLL: Duration = Duration::from_millis(200);
    const DEVTOOLS_TIMEOUT: Duration = Duration::from_secs(30);
    const CMD_TIMEOUT: Duration = Duration::from_secs(30);

    // ── CDP client core ──────────────────────────────────────────────────────

    #[derive(Clone)]
    struct CdpClient {
        inner: Arc<CdpInner>,
    }

    struct CdpInner {
        outbox: mpsc::UnboundedSender<String>,
        pending: Mutex<HashMap<u64, oneshot::Sender<Result<Value, String>>>>,
        next_id: AtomicU64,
    }

    impl CdpClient {
        fn new(outbox: mpsc::UnboundedSender<String>) -> Self {
            Self {
                inner: Arc::new(CdpInner {
                    outbox,
                    pending: Mutex::new(HashMap::new()),
                    next_id: AtomicU64::new(0),
                }),
            }
        }

        /// Send a CDP command and await its response (correlated by id).
        async fn command(&self, method: &str, params: Value) -> Result<Value, String> {
            let id = self.inner.next_id.fetch_add(1, Ordering::SeqCst) + 1;
            let payload = json!({ "id": id, "method": method, "params": params });
            let (tx, rx) = oneshot::channel();
            self.inner.pending.lock().await.insert(id, tx);
            let raw =
                serde_json::to_string(&payload).map_err(|e| format!("cdp encode: {e}"))?;
            self.inner
                .outbox
                .send(raw)
                .map_err(|_| "cdp writer closed".to_string())?;
            let result = match tokio::time::timeout(CMD_TIMEOUT, rx).await {
                Ok(Ok(v)) => v?,
                Ok(Err(_)) => return Err("cdp response channel closed".into()),
                Err(_) => {
                    self.inner.pending.lock().await.remove(&id);
                    return Err(format!("cdp command '{method}' timed out"));
                }
            };
            Ok(result)
        }

        /// `Runtime.evaluate` with `returnByValue`; returns the JS value (or the
        /// exception message on throw). The workhorse — most cmd_* below are
        /// thin wrappers over this.
        async fn evaluate(&self, expression: &str) -> Result<Value, String> {
            let resp = self
                .command(
                    "Runtime.evaluate",
                    json!({
                        "expression": expression,
                        "returnByValue": true,
                        "awaitPromise": true,
                        "userGesture": true,
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
                return Err(msg.to_string());
            }
            let result = resp.get("result").cloned().unwrap_or(Value::Null);
            Ok(result.get("value").cloned().unwrap_or(Value::Null))
        }
    }

    // ── event/payload helpers ────────────────────────────────────────────────

    /// Render a CDP `RemoteObject` (e.g. a `console.log` argument) to a string.
    fn remote_object_text(o: &Value) -> String {
        if let Some(val) = o.get("value") {
            match val {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            }
        } else if let Some(d) = o.get("description").and_then(|d| d.as_str()) {
            d.to_string()
        } else if let Some(t) = o.get("type").and_then(|t| t.as_str()) {
            format!("[{t}]")
        } else {
            String::new()
        }
    }

    /// Convert a CDP timestamp to an RFC3339 string; falls back to "now" if
    /// absent. CDP emits some timestamps in seconds-since-epoch and others in
    /// milliseconds-since-epoch (the Runtime.consoleAPICalled/exceptionThrown
    /// ones are ms); auto-detect so both render sanely.
    fn cdp_ts(ts: Option<f64>) -> String {
        let raw = match ts {
            Some(t) => t,
            None => return chrono::Utc::now().to_rfc3339(),
        };
        let secs = if raw > 4_000_000_000.0 { raw / 1000.0 } else { raw };
        let whole = secs.floor() as i64;
        let nanos = ((secs - secs.floor()) * 1e9) as u32;
        chrono::DateTime::from_timestamp(whole, nanos)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
    }

    /// Keep a buffer bounded so a chatty page can't grow it without limit.
    fn truncate_vec<T>(v: &mut Vec<T>, cap: usize) {
        if v.len() > cap {
            v.drain(0..v.len() - cap);
        }
    }

    // ── launch + connect ─────────────────────────────────────────────────────

    pub(super) async fn spawn_browser(
        base_url: String,
        _initial_url: Option<String>,
        console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
        errors: Arc<RwLock<Vec<ErrorEntry>>>,
    ) -> Result<BrowserHandle, String> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<BrowserCommand>(64);
        let connected = Arc::new(RwLock::new(false));

        let exe = resolve_executable()?;
        let port = pick_free_port().ok_or_else(|| "no free port for devtools".to_string())?;

        let child: Child = Command::new(&exe)
            .args([
                "--headless=new",
                "--no-sandbox",
                "--disable-dev-shm-usage",
                "--disable-gpu",
                "--disable-extensions",
                "--disable-background-networking",
                "--no-first-run",
                &format!("--remote-debugging-port={port}"),
                &format!("--window-size={DEFAULT_VIEWPORT_W},{DEFAULT_VIEWPORT_H}"),
                &base_url,
            ])
            // Keep chrome's stdio off our pipes (it would otherwise wedge the
            // owning shell once orphaned).
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| format!("failed to launch chrome ({exe}): {e}"))?;

        // Wait for the devtools HTTP endpoint, then read the browser ws URL.
        let ws_url = wait_for_devtools(port).await?;

        let (ws, _resp) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| format!("devtools ws connect failed: {e}"))?;
        crate::log_ok!("Debug browser connected (chromium raw-CDP)");

        let (mut sink, mut stream) = ws.split();
        let (outbox, mut inbox) = mpsc::unbounded_channel::<String>();
        let client = CdpClient::new(outbox);

        // Writer: drain outbound JSON → ws sink.
        tokio::spawn(async move {
            while let Some(raw) = inbox.recv().await {
                if sink.send(Message::Text(raw.into())).await.is_err() {
                    break;
                }
            }
            let _ = sink.send(Message::Close(None)).await;
        });

        // Reader: dispatch command responses by id; capture the few events we
        // actually need (console output + uncaught exceptions); ignore
        // everything else. Handling known events while dropping unknown ones
        // is precisely what keeps this client skew-proof across Chrome
        // versions — a brand-new event variant never crashes the deserializer.
        let pending = client.inner.clone();
        let conn_reader = connected.clone();
        let console_buf = console_log;
        let error_buf = errors;
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                let text = match msg {
                    Ok(Message::Text(t)) => t.to_string(),
                    Ok(Message::Ping(_)) => continue, // auto-ponged by the runtime
                    Ok(_) => continue,
                    Err(_) => break,
                };
                let v: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue, // not JSON / partial — ignore
                };
                // Command response — resolve by id.
                if let Some(id) = v.get("id").and_then(|i| i.as_u64()) {
                    if let Some(tx) = pending.pending.lock().await.remove(&id) {
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
                    continue;
                }
                // Event — handle the ones we need, ignore the rest.
                let method = match v.get("method").and_then(|m| m.as_str()) {
                    Some(m) => m,
                    None => continue,
                };
                match method {
                    "Runtime.consoleAPICalled" => {
                        if let Some(p) = v.get("params") {
                            let level = p
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("log")
                                .to_string();
                            let text = p
                                .get("args")
                                .and_then(|a| a.as_array())
                                .map(|args| {
                                    args.iter().map(remote_object_text).collect::<Vec<_>>().join(" ")
                                })
                                .unwrap_or_default();
                            let mut buf = console_buf.write().await;
                            buf.push(ConsoleEntry {
                                level,
                                text,
                                timestamp: cdp_ts(
                                    p.get("timestamp").and_then(|t| t.as_f64()),
                                ),
                                source: Some("runtime".into()),
                            });
                            truncate_vec(&mut buf, 500);
                        }
                    }
                    "Runtime.exceptionThrown" => {
                        if let Some(ed) =
                            v.get("params").and_then(|p| p.get("exceptionDetails"))
                        {
                            let message = ed
                                .get("exception")
                                .and_then(|e| e.get("description"))
                                .and_then(|d| d.as_str())
                                .or_else(|| ed.get("text").and_then(|t| t.as_str()))
                                .unwrap_or("uncaught exception")
                                .to_string();
                            let stack = ed
                                .get("stackTrace")
                                .and_then(|s| s.get("callFrames"))
                                .and_then(|f| f.as_array())
                                .map(|frames| {
                                    frames
                                        .iter()
                                        .filter_map(|f| {
                                            let fn_ = f
                                                .get("functionName")
                                                .and_then(|x| x.as_str())
                                                .unwrap_or("<anon>");
                                            let url = f
                                                .get("url")
                                                .and_then(|x| x.as_str())
                                                .unwrap_or("");
                                            let ln = f
                                                .get("lineNumber")
                                                .and_then(|x| x.as_i64())
                                                .unwrap_or(0);
                                            let co = f
                                                .get("columnNumber")
                                                .and_then(|x| x.as_i64())
                                                .unwrap_or(0);
                                            Some(format!("    at {fn_} ({url}:{ln}:{co})"))
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n")
                                });
                            let mut buf = error_buf.write().await;
                            buf.push(ErrorEntry {
                                message,
                                stack,
                                r#type: "exception".into(),
                                timestamp: cdp_ts(
                                    v.get("params")
                                        .and_then(|p| p.get("timestamp"))
                                        .and_then(|t| t.as_f64()),
                                ),
                            });
                            truncate_vec(&mut buf, 500);
                        }
                    }
                    _ => {} // unknown event — deliberately ignored (skew-proof)
                }
            }
            *conn_reader.write().await = false;
        });

        // Enable the domains we use.
        let _ = client.command("Page.enable", json!({})).await;
        let _ = client.command("Runtime.enable", json!({})).await;
        let _ = client.command("Network.enable", json!({})).await;

        *connected.write().await = true;

        // Per-command dispatch loop. Holds the child so chrome is reaped when
        // every BrowserHandle (and thus cmd_rx) is dropped.
        tokio::spawn(async move {
            let client = client;
            while let Some(cmd) = cmd_rx.recv().await {
                let c = client.clone();
                tokio::spawn(async move {
                    dispatch_command(&c, cmd).await;
                });
            }
            drop(child); // kill_on_drop reaps chrome here.
        });

        Ok(BrowserHandle {
            tx: cmd_tx,
            connected,
        })
    }

    async fn wait_for_devtools(port: u16) -> Result<String, String> {
        // We need a PAGE-level devtools endpoint: the browser-level ws served
        // by /json/version only handles Target.*/Browser.*, and rejects
        // Page.*/Runtime.* with "<method> wasn't found". Poll /json/list for
        // the first page target's websocket URL instead.
        let list_url = format!("http://127.0.0.1:{port}/json/list");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let deadline = std::time::Instant::now() + DEVTOOLS_TIMEOUT;
        loop {
            if std::time::Instant::now() > deadline {
                return Err(format!("devtools never came up on :{port}"));
            }
            if let Ok(resp) = client.get(&list_url).send().await {
                if resp.status().is_success() {
                    if let Ok(Value::Array(targets)) = resp.json::<Value>().await {
                        for t in &targets {
                            if t.get("type").and_then(|v| v.as_str()) == Some("page") {
                                if let Some(ws) = t
                                    .get("webSocketDebuggerUrl")
                                    .and_then(|w| w.as_str())
                                    .map(|s| s.to_string())
                                {
                                    return Ok(ws);
                                }
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(DEVTOOLS_POLL).await;
        }
    }

    fn pick_free_port() -> Option<u16> {
        std::net::TcpListener::bind(("127.0.0.1", 0))
            .ok()
            .and_then(|l| l.local_addr().ok())
            .map(|a| a.port())
    }

    fn resolve_executable() -> Result<String, String> {
        if let Ok(exe) = std::env::var("CHROME_PATH") {
            if !exe.is_empty() {
                return Ok(exe);
            }
        }
        if let Ok(exe) = which_chromium() {
            return Ok(exe);
        }
        Err(
            "no chrome/chromium found. Set CHROME_PATH or install chromium on PATH."
                .to_string(),
        )
    }

    fn which_chromium() -> Result<String, ()> {
        let candidates = [
            "chromium-browser",
            "chromium",
            "google-chrome",
            "google-chrome-stable",
            "chrome",
        ];
        for name in &candidates {
            if let Ok(output) = std::process::Command::new("which").arg(name).output() {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
            }
        }
        Err(())
    }

    // ── command dispatch ─────────────────────────────────────────────────────

    async fn dispatch_command(client: &CdpClient, cmd: BrowserCommand) {
        match cmd {
            BrowserCommand::Navigate { url, wait_for, resp } => {
                let r = cmd_navigate(client, &url, wait_for.as_deref()).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Screenshot { selector, full_page, resp } => {
                let r = cmd_screenshot(client, selector.as_deref(), full_page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Click { selector, resp } => {
                let r = cmd_click(client, &selector).await;
                let _ = resp.send(r);
            }
            BrowserCommand::TypeText { selector, text, clear_first, submit, resp } => {
                let r = cmd_type(client, &selector, &text, clear_first, submit).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Evaluate { expression, await_promise, resp } => {
                let r = cmd_evaluate(client, &expression, await_promise).await;
                let _ = resp.send(r);
            }
            BrowserCommand::DomQuery { selector, attribute, computed, resp } => {
                let r = cmd_dom_query(client, &selector, attribute.as_deref(), computed.as_deref()).await;
                let _ = resp.send(r);
            }
            BrowserCommand::IsReady { resp } => {
                let r = cmd_is_ready(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Press { key, resp, .. } => {
                let r = cmd_press(client, &key).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Scroll { selector, x, y, resp } => {
                let r = cmd_scroll(client, selector.as_deref(), x, y).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Resize { width, height, resp } => {
                let r = cmd_resize(client, width, height).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Viewport { resp } => {
                let r = cmd_viewport(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::A11y { selector, depth, resp } => {
                let r = cmd_a11y(client, selector.as_deref(), depth).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Network { resp } => {
                let r = cmd_network(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Performance { resp } => {
                let r = cmd_performance(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Drag { from_selector, to_selector, steps, resp } => {
                let r = cmd_drag(client, &from_selector, &to_selector, steps).await;
                let _ = resp.send(r);
            }
            BrowserCommand::WebSocket { resp } => {
                let r = cmd_websocket(client).await;
                let _ = resp.send(r);
            }
        }
    }

    async fn cmd_navigate(
        client: &CdpClient,
        url: &str,
        wait_for: Option<&str>,
    ) -> Result<NavigateResponse, String> {
        let resp = client
            .command("Page.navigate", json!({ "url": url }))
            .await
            .map_err(|e| format!("navigate: {e}"))?;
        if let Some(err) = resp.get("errorText").and_then(|t| t.as_str()) {
            return Err(format!("navigate: {err}"));
        }
        if matches!(wait_for, Some("hydration") | Some("ready")) {
            tokio::time::sleep(Duration::from_secs(3)).await;
        } else if matches!(wait_for, Some("load")) {
            tokio::time::sleep(Duration::from_millis(500)).await;
        } else {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let title = client
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        Ok(NavigateResponse { url: url.to_string(), title })
    }

    async fn cmd_screenshot(
        client: &CdpClient,
        selector: Option<&str>,
        full_page: bool,
    ) -> Result<ScreenshotResponse, String> {
        if let Some(sel) = selector {
            let rect_js = format!(
                r#"(() => {{ const e = document.querySelector({sel:?}); if (!e) throw 'element not found'; const r = e.getBoundingClientRect(); const dpr = window.devicePixelRatio || 1; return JSON.stringify({{ x: r.x, y: r.y, width: r.width, height: r.height, scale: dpr }}); }})()"#,
            );
            let raw = client.evaluate(&rect_js).await.map_err(|e| format!("screenshot rect: {e}"))?;
            let s = raw.as_str().ok_or_else(|| "screenshot rect: non-string".to_string())?;
            let rect: Value = serde_json::from_str(s).map_err(|e| format!("screenshot rect parse: {e}"))?;
            let clip = json!({
                "x": rect["x"], "y": rect["y"],
                "width": rect["width"], "height": rect["height"],
                "scale": rect["scale"],
            });
            let resp = client
                .command("Page.captureScreenshot", json!({ "format": "png", "clip": clip }))
                .await
                .map_err(|e| format!("screenshot element: {e}"))?;
            return screenshot_response_from(&resp);
        }
        let params = if full_page {
            json!({ "format": "png", "captureBeyondViewport": true, "fromSurface": true })
        } else {
            json!({ "format": "png" })
        };
        let resp = client
            .command("Page.captureScreenshot", params)
            .await
            .map_err(|e| format!("screenshot: {e}"))?;
        screenshot_response_from(&resp)
    }

    fn screenshot_response_from(resp: &Value) -> Result<ScreenshotResponse, String> {
        let data = resp
            .get("data")
            .and_then(|d| d.as_str())
            .ok_or_else(|| "screenshot: no data".to_string())?
            .to_string();
        Ok(ScreenshotResponse {
            data,
            mime_type: "image/png".into(),
            width: DEFAULT_VIEWPORT_W,
            height: DEFAULT_VIEWPORT_H,
        })
    }

    async fn cmd_click(client: &CdpClient, selector: &str) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ const el = document.querySelector({selector:?}); if (!el) throw 'element not found'; el.scrollIntoView({{ block: 'center' }}); el.click(); }})()"#,
        );
        client.evaluate(&js).await.map_err(|e| format!("click: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_type(
        client: &CdpClient,
        selector: &str,
        text: &str,
        clear_first: bool,
        _submit: bool,
    ) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ const el = document.querySelector({selector:?}); if (!el) throw 'element not found'; el.focus(); if ({clear}) {{ el.value = ''; }} else {{ el.value = el.value; }} el.value += {text:?}; el.dispatchEvent(new Event('input', {{ bubbles: true }})); el.dispatchEvent(new Event('change', {{ bubbles: true }})); }})()"#,
            clear = clear_first,
            text = text,
        );
        client.evaluate(&js).await.map_err(|e| format!("type: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_evaluate(
        client: &CdpClient,
        expression: &str,
        await_promise: bool,
    ) -> Result<EvaluateResponse, String> {
        let resp = client
            .command(
                "Runtime.evaluate",
                json!({
                    "expression": expression,
                    "returnByValue": true,
                    "awaitPromise": await_promise,
                    "userGesture": true,
                }),
            )
            .await
            .map_err(|e| format!("evaluate: {e}"))?;
        if let Some(exc) = resp.get("exceptionDetails") {
            let msg = exc
                .get("exception")
                .and_then(|e| e.get("description"))
                .and_then(|d| d.as_str())
                .or_else(|| exc.get("text").and_then(|t| t.as_str()))
                .unwrap_or("runtime exception");
            return Err(msg.to_string());
        }
        let val = resp
            .get("result")
            .and_then(|r| r.get("value"))
            .cloned()
            .unwrap_or(Value::Null);
        let type_name = match &val {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) | Value::Object(_) => "object",
        };
        Ok(EvaluateResponse { result: val, r#type: type_name.into() })
    }

    async fn cmd_dom_query(
        client: &CdpClient,
        selector: &str,
        attribute: Option<&str>,
        _computed: Option<&[String]>,
    ) -> Result<DomNodeResponse, String> {
        if let Some(attr) = attribute {
            let js = format!(
                "(() => {{ const el = document.querySelector({sel:?}); if (!el) return null; return el.getAttribute({attr:?}); }})()",
                sel = selector,
                attr = attr,
            );
            let val = client.evaluate(&js).await.map_err(|e| format!("dom query: {e}"))?;
            let r = val.as_str().map(|s| s.to_string());
            let count = if r.is_some() { 1 } else { 0 };
            return Ok(DomNodeResponse {
                tag: None, text: r, html: None, attributes: None,
                visible: None, count, rect: None, computed: None,
            });
        }
        let js = format!(
            r#"(() => {{ const els = document.querySelectorAll({sel:?}); if (!els.length) throw 'not found'; const el = els[0]; const r = el.getBoundingClientRect(); return JSON.stringify({{ tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 2000), html: el.outerHTML.substring(0, 5000), attrs: Object.fromEntries(Array.from(el.attributes).map(a => [a.name, a.value])), visible: r.width > 0 && r.height > 0, count: els.length, rect: {{ x: r.x, y: r.y, width: r.width, height: r.height }} }}); }})()"#,
            sel = selector,
        );
        let val = client.evaluate(&js).await.map_err(|e| format!("dom query: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "dom query: non-string result".to_string())?;
        serde_json::from_str::<DomNodeResponse>(json_str)
            .map_err(|e| format!("dom query deserialize: {e}"))
    }

    async fn cmd_is_ready(client: &CdpClient) -> Result<ReadyResponse, String> {
        let js = r#"(() => { const w = !!globalThis.__wasmExports; const h = document.documentElement.dataset.tairitsuReady === 'hydrated'; return JSON.stringify({ ready: w && h, wasm_loaded: w, hydrated: h, url: location.href }); })()"#;
        let val = client.evaluate(js).await.map_err(|e| format!("is_ready: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "is_ready: non-string".to_string())?;
        serde_json::from_str::<ReadyResponse>(json_str).map_err(|e| format!("is_ready deserialize: {e}"))
    }

    async fn cmd_press(client: &CdpClient, key: &str) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ document.dispatchEvent(new KeyboardEvent('keydown', {{key: {key:?}, code: {key:?}, bubbles: true}})); document.dispatchEvent(new KeyboardEvent('keyup', {{key: {key:?}, code: {key:?}, bubbles: true}})); }})()"#,
        );
        client.evaluate(&js).await.map_err(|e| format!("press: {e}"))?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    async fn cmd_scroll(
        client: &CdpClient,
        selector: Option<&str>,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
        let js = if let Some(sel) = selector {
            format!(r#"(() => {{ const el = document.querySelector({sel:?}); if (el) el.scrollBy({x}, {y}); }})()"#)
        } else {
            format!(r#"window.scrollBy({x}, {y})"#)
        };
        client.evaluate(&js).await.map_err(|e| format!("scroll: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_resize(client: &CdpClient, width: u32, height: u32) -> Result<(), String> {
        client
            .command(
                "Emulation.setDeviceMetricsOverride",
                json!({ "width": width, "height": height, "deviceScaleFactor": 1, "mobile": false }),
            )
            .await
            .map_err(|e| format!("resize: {e}"))?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    async fn cmd_viewport(client: &CdpClient) -> Result<ViewportResponse, String> {
        let js = r#"(() => { const dpr = window.devicePixelRatio || 1; return JSON.stringify({ width: window.innerWidth, height: window.innerHeight, device_pixel_ratio: dpr }); })()"#;
        let val = client.evaluate(js).await.map_err(|e| format!("viewport: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "viewport: non-string".to_string())?;
        serde_json::from_str::<ViewportResponse>(json_str).map_err(|e| format!("viewport deserialize: {e}"))
    }

    async fn cmd_a11y(
        client: &CdpClient,
        selector: Option<&str>,
        depth: u32,
    ) -> Result<Vec<A11yNode>, String> {
        let sel_js = match selector {
            Some(s) => format!("document.querySelector({s:?})"),
            None => "document.body".to_string(),
        };
        let js_body = r#"
(function(){
function getA11y(el,d,maxD){
if(!el||d>maxD)return null;
var tagRoles={BUTTON:'button',SELECT:'listbox',OPTION:'option',A:'link',H1:'heading',H2:'heading',H3:'heading',H4:'heading',H5:'heading',H6:'heading',NAV:'navigation',MAIN:'main',HEADER:'banner',FOOTER:'contentinfo',ASIDE:'complementary',FORM:'form',TABLE:'table',UL:'list',OL:'list',LI:'listitem',IMG:'img',SVG:'img',PROGRESS:'progressbar',METER:'meter',DIALOG:'dialog',DETAILS:'group',SUMMARY:'button',FIELDSET:'group'};
var inputRoles={checkbox:'checkbox',radio:'radio'};
var role=el.getAttribute('role')||(el.tagName?(tagRoles[el.tagName]||(el.tagName==='INPUT'?(inputRoles[el.getAttribute('type')]||'textbox'):(el.tagName==='TEXTAREA'?'textbox':undefined))):undefined);
var name=el.getAttribute('aria-label')||el.getAttribute('title')||((el.tagName==='INPUT'||el.tagName==='TEXTAREA')?el.getAttribute('placeholder'):null)||(el.tagName==='IMG'?el.getAttribute('alt'):null)||null;
var desc=el.getAttribute('aria-description')||null;
var states=[];
if(el.disabled)states.push('disabled');
if(el.getAttribute('aria-hidden')==='true')states.push('hidden');
if(el.getAttribute('aria-expanded')==='true')states.push('expanded');
if(el.getAttribute('aria-expanded')==='false')states.push('collapsed');
if(el.getAttribute('aria-selected')==='true')states.push('selected');
if(el.getAttribute('aria-checked')==='true')states.push('checked');
if(el.getAttribute('aria-checked')==='mixed')states.push('mixed');
var children=[];
if(d<maxD){for(var i=0;i<el.children.length;i++){var child=getA11y(el.children[i],d+1,maxD);if(child)children.push(child)}}
return{name:name,role:role||null,description:desc,states:states,tag:el.tagName?el.tagName.toLowerCase():null,children:children}
}
var root=SEL_JS;
if(!root)throw'element not found';
var tree=getA11y(root,0,DEPTH);
return JSON.stringify([tree])
})()
"#.replace("SEL_JS", &sel_js).replace("DEPTH", &depth.to_string());
        let val = client.evaluate(&js_body).await.map_err(|e| format!("a11y: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "a11y: non-string".to_string())?;
        serde_json::from_str::<Vec<A11yNode>>(json_str).map_err(|e| format!("a11y deserialize: {e}"))
    }

    async fn cmd_network(client: &CdpClient) -> Result<NetworkResponse, String> {
        let js = r#"(() => { var entries = performance.getEntriesByType('resource').slice(0, 100).map(function(e) { return { name: e.name, type: e.initiatorType || 'unknown', duration: Math.round(e.duration * 100) / 100, size: e.transferSize || 0, url: e.name }; }); return JSON.stringify({ resources: entries }); })()"#;
        let val = client.evaluate(js).await.map_err(|e| format!("network: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "network: non-string".to_string())?;
        serde_json::from_str::<NetworkResponse>(json_str).map_err(|e| format!("network deserialize: {e}"))
    }

    async fn cmd_performance(client: &CdpClient) -> Result<PerformanceMetrics, String> {
        let js = r#"(() => { var nav = performance.getEntriesByType('navigation')[0] || {}; var fcp = null; try { fcp = performance.getEntriesByName('first-contentful-paint')[0].startTime || null; } catch(e) {} var dn = document.querySelectorAll('*').length; var heap = null; try { heap = Math.round((performance.memory ? performance.memory.usedJSHeapSize : 0) / 1048576 * 100) / 100; } catch(e) {} return JSON.stringify({ dom_content_loaded_ms: Math.round((nav.domContentLoadedEventEnd - nav.startTime) * 100) / 100 || null, dom_complete_ms: Math.round((nav.domComplete - nav.startTime) * 100) / 100 || null, load_event_ms: Math.round((nav.loadEventEnd - nav.startTime) * 100) / 100 || null, fcp_ms: fcp ? Math.round(fcp * 100) / 100 : null, lcp_ms: null, cls: null, dom_nodes: dn, js_heap_used_mb: heap, wasm_loaded: !!globalThis.__wasmExports, hydrated: document.documentElement.dataset.tairitsuReady === 'hydrated', timestamp: new Date().toISOString() }); })()"#;
        let val = client.evaluate(js).await.map_err(|e| format!("performance: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "performance: non-string".to_string())?;
        serde_json::from_str::<PerformanceMetrics>(json_str).map_err(|e| format!("performance deserialize: {e}"))
    }

    async fn cmd_drag(
        client: &CdpClient,
        from_selector: &str,
        to_selector: &str,
        steps: u32,
    ) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ var src = document.querySelector({from:?}); var dst = document.querySelector({to:?}); if (!src || !dst) throw 'element not found'; var sr = src.getBoundingClientRect(); var dr = dst.getBoundingClientRect(); var sx = sr.x + sr.width/2, sy = sr.y + sr.height/2; var dx = dr.x + dr.width/2, dy = dr.y + dr.height/2; src.dispatchEvent(new MouseEvent('mousedown', {{clientX: sx, clientY: sy, bubbles: true}})); for (var i = 1; i <= {steps}; i++) {{ var t = i/{steps}; var cx = sx + (dx - sx)*t, cy = sy + (dy - sy)*t; document.dispatchEvent(new MouseEvent('mousemove', {{clientX: cx, clientY: cy, bubbles: true}})); }} dst.dispatchEvent(new MouseEvent('mouseup', {{clientX: dx, clientY: dy, bubbles: true}})); dst.dispatchEvent(new MouseEvent('drop', {{clientX: dx, clientY: dy, bubbles: true}})); }})()"#,
            from = from_selector, to = to_selector, steps = steps,
        );
        client.evaluate(&js).await.map_err(|e| format!("drag: {e}"))?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    async fn cmd_websocket(client: &CdpClient) -> Result<WebSocketInfo, String> {
        let js = r#"(() => { var c = 0; var conns = []; var t = window._wsTracker || []; t.forEach(function(ws) { c++; conns.push({ url: ws.url || 'unknown', state: ws.readyState === 0 ? 'connecting' : ws.readyState === 1 ? 'open' : ws.readyState === 2 ? 'closing' : 'closed', created_at_ms: null }); }); return JSON.stringify({ active_count: c, connections: conns }); })()"#;
        let val = client.evaluate(js).await.map_err(|e| format!("websocket: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "websocket: non-string".to_string())?;
        serde_json::from_str::<WebSocketInfo>(json_str).map_err(|e| format!("websocket deserialize: {e}"))
    }
}

// ── DebugState ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct DebugState {
    dist_dir: String,
    package_name: String,
    dev_port: u16,
    debug_port: u16,
    start_time: Instant,
    base_url: String,
    console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    errors: Arc<RwLock<Vec<ErrorEntry>>>,
    rejections: Arc<RwLock<Vec<ErrorEntry>>>,
    browser: Option<Arc<BrowserHandle>>,
    browser_engine: String,
}

impl DebugState {
    fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

// ── Server startup ───────────────────────────────────────────────────────

/// Inputs needed to launch the standalone debug API + browser. Carries only
/// what the debug surface actually uses — deliberately decoupled from the
/// full app [`Config`](crate::config::Config) so the debug server can run
/// without a tairitsu app project (see the `tairitsu debug` subcommand).
#[derive(Debug, Clone)]
pub struct DebugServerConfig {
    /// URL the browser opens on launch, and the base used to resolve any
    /// relative path passed to `/navigate`.
    pub base_url: String,
    /// Informational only (surfaced via `/info`): the app dev-server port, or
    /// 0 when running standalone (`tairitsu debug`).
    pub dev_port: u16,
    /// Informational only (surfaced via `/info`): the build output dir label.
    pub dist_dir: String,
    /// Informational only (surfaced via `/info`): the package name label.
    pub package_name: String,
}

pub async fn start_debug_server(cfg: DebugServerConfig, debug_port: u16) -> crate::Result<()> {
    let base_url = cfg.base_url.clone();
    let dev_port = cfg.dev_port;
    let console_log = Arc::new(RwLock::new(Vec::new()));
    let errors = Arc::new(RwLock::new(Vec::new()));

    #[cfg(feature = "debug-browser")]
    let (browser, browser_engine) = {
        crate::log_info!("Debug browser engine: chromium (headless CDP)");
        match tokio::time::timeout(
            Duration::from_secs(30),
            engine::spawn_browser(base_url.clone(), None, console_log.clone(), errors.clone()),
        )
        .await
        {
            Ok(Ok(b)) => (Some(Arc::new(b)), "chromium".to_string()),
            Ok(Err(e)) => {
                crate::log_fail!("[debug-browser] Failed: {e}");
                (None, "none".to_string())
            }
            Err(_) => {
                crate::log_fail!("[debug-browser] Timed out after 30s");
                (None, "none".to_string())
            }
        }
    };
    #[cfg(not(feature = "debug-browser"))]
    let (browser, browser_engine): (Option<Arc<BrowserHandle>>, String) = (None, "none".into());

    let browser_engine = if browser.is_some() {
        browser_engine
    } else {
        "none".into()
    };

    let state = DebugState {
        dist_dir: cfg.dist_dir.clone(),
        package_name: cfg.package_name.clone(),
        dev_port,
        debug_port,
        base_url,
        console_log,
        errors,
        rejections: Arc::new(RwLock::new(Vec::new())),
        browser,
        browser_engine,
        start_time: Instant::now(),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], debug_port));
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/ready", get(ready_handler))
        .route("/navigate", post(navigate_handler))
        .route("/screenshot", post(screenshot_handler))
        .route("/click", post(click_handler))
        .route("/type", post(type_handler))
        .route("/press", post(press_handler))
        .route("/scroll", post(scroll_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/console", get(console_handler))
        .route("/console", delete(console_clear_handler))
        .route("/dom", get(dom_query_handler))
        .route("/dom/computed", post(computed_style_handler))
        .route("/viewport", get(viewport_handler))
        .route("/resize", post(resize_handler))
        .route("/errors", get(errors_handler))
        .route("/drag", post(drag_handler))
        .route("/a11y", get(a11y_handler))
        .route("/batch", post(batch_handler))
        .route("/network", get(network_handler))
        .route("/performance", get(performance_handler))
        .route("/websocket", get(websocket_handler))
        .route("/source-map", post(source_map_handler))
        .layer(CompressionLayer::new())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    crate::log_ok!(
        "Debug API v{} listening on http://localhost:{}",
        DEBUG_API_VERSION,
        debug_port
    );
    crate::log_info!(
        "Endpoints: /health /info /ready /navigate /screenshot /click /type /press /scroll /evaluate /console /dom /dom/computed /viewport /resize /errors /drag /a11y /batch /network /performance /websocket /source-map"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── HTTP handlers ─────────────────────────────────────────────────────────

async fn health_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        version: crate::VERSION.into(),
        api_version: DEBUG_API_VERSION.into(),
        uptime_secs: state.uptime_secs(),
    }))
}

async fn info_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let bc = state
        .browser
        .as_ref()
        .is_some_and(|b| futures::executor::block_on(b.is_connected()));
    ResponseJson(ApiResponse::ok(InfoResponse {
        version: crate::VERSION.into(),
        api_version: DEBUG_API_VERSION.into(),
        dev_port: state.dev_port,
        debug_port: state.debug_port,
        dist_dir: state.dist_dir.clone(),
        package_name: state.package_name.clone(),
        pid: std::process::id(),
        started_at_iso: chrono::Utc::now().to_rfc3339(),
        uptime_secs: state.uptime_secs(),
        browser_connected: bc,
        browser_engine: state.browser_engine.clone(),
        viewport: [DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H],
    }))
}

async fn ready_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<ReadyResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::IsReady { resp: tx }).await.is_err() {
        return chan_closed::<ReadyResponse>();
    }
    await_op(rx).await
}

async fn navigate_handler(
    State(state): State<DebugState>,
    Json(req): Json<NavigateRequest>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<NavigateResponse>(),
    };
    let target = if req.url.starts_with("http") {
        req.url
    } else {
        format!("{}{}", state.base_url, req.url)
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Navigate {
            url: target,
            wait_for: req.wait_for,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<NavigateResponse>();
    }
    await_op(rx).await
}

async fn screenshot_handler(
    State(state): State<DebugState>,
    Json(params): Json<ScreenshotParams>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<ScreenshotResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Screenshot {
            selector: params.selector,
            full_page: params.full_page.unwrap_or(false),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<ScreenshotResponse>();
    }
    await_op(rx).await
}

async fn click_handler(
    State(state): State<DebugState>,
    Json(req): Json<ClickRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Click {
            selector: req.selector,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn type_handler(
    State(state): State<DebugState>,
    Json(req): Json<TypeRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::TypeText {
            selector: req.selector,
            text: req.text,
            clear_first: req.clear_first.unwrap_or(true),
            submit: req.submit.unwrap_or(false),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn press_handler(
    State(state): State<DebugState>,
    Json(req): Json<PressRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Press {
            key: req.key,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn scroll_handler(
    State(state): State<DebugState>,
    Json(req): Json<ScrollRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (tx, rx) = oneshot::channel();
    let (x, y) = match req.direction.as_deref() {
        Some("up") => (0.0, -(req.amount.unwrap_or(300.0))),
        Some("down") => (0.0, req.amount.unwrap_or(300.0)),
        Some("left") => (-(req.amount.unwrap_or(300.0)), 0.0),
        Some("right") => (req.amount.unwrap_or(300.0), 0.0),
        _ => (req.x.unwrap_or(0.0), req.y.unwrap_or(0.0)),
    };
    if br
        .send(BrowserCommand::Scroll {
            selector: req.selector,
            x,
            y,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn evaluate_handler(
    State(state): State<DebugState>,
    Json(req): Json<EvaluateRequest>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<EvaluateResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Evaluate {
            expression: req.expression,
            await_promise: req.await_promise.unwrap_or(false),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<EvaluateResponse>();
    }
    await_op(rx).await
}

async fn console_handler(
    State(state): State<DebugState>,
    Query(params): Query<ConsoleQueryParams>,
) -> impl IntoResponse {
    let entries = state.console_log.read().await;
    let mut filtered: Vec<ConsoleEntry> = entries
        .iter()
        .filter(|e| {
            if let Some(ref levels) = params.level {
                let allowed: Vec<&str> = levels.split(',').collect();
                if !allowed.contains(&e.level.as_str()) {
                    return false;
                }
            }
            if let Some(ref src) = params.source {
                if e.source.as_deref() != Some(src.as_str()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();
    if let Some(limit) = params.limit {
        filtered.truncate(limit);
    }
    ResponseJson(ApiResponse::ok(ConsoleResponse { entries: filtered }))
}

async fn console_clear_handler(State(state): State<DebugState>) -> impl IntoResponse {
    state.console_log.write().await.clear();
    ResponseJson(ApiResponse::ok(serde_json::json!({"cleared": true})))
}

async fn dom_query_handler(
    State(state): State<DebugState>,
    Query(params): Query<DomQueryParams>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<DomNodeResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::DomQuery {
            selector: params.selector,
            attribute: params.attribute,
            computed: None,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<DomNodeResponse>();
    }
    await_op(rx).await
}

async fn computed_style_handler(
    State(state): State<DebugState>,
    Json(params): Json<ComputedStyleParams>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<ComputedStyleResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::DomQuery {
            selector: params.selector.clone(),
            attribute: None,
            computed: params.properties,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<ComputedStyleResponse>();
    }
    match tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await {
        Ok(Ok(Ok(dom))) => {
            let computed = dom.computed.unwrap_or_default();
            (
                StatusCode::OK,
                ResponseJson(ApiResponse::ok(ComputedStyleResponse {
                    selector: params.selector,
                    properties: computed,
                })),
            )
        }
        Ok(Ok(Err(e))) => (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::err(e))),
        Ok(Err(_)) => chan_closed::<ComputedStyleResponse>(),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            ResponseJson(ApiResponse::err("Operation timed out")),
        ),
    }
}

async fn viewport_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<ViewportResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Viewport { resp: tx })
        .await
        .is_err()
    {
        return chan_closed::<ViewportResponse>();
    }
    await_op(rx).await
}

async fn resize_handler(
    State(state): State<DebugState>,
    Json(req): Json<ResizeRequest>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (w, h) = match req.preset.as_deref() {
        Some("mobile") => (375, 812),
        Some("tablet") => (768, 1024),
        Some("desktop") => (1280, 720),
        Some("wide") => (1920, 1080),
        _ => (
            req.width.unwrap_or(DEFAULT_VIEWPORT_W),
            req.height.unwrap_or(DEFAULT_VIEWPORT_H),
        ),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Resize {
            width: w,
            height: h,
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn errors_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(ErrorsResponse {
        errors: state.errors.read().await.clone(),
        unhandled_rejections: state.rejections.read().await.clone(),
    }))
}

async fn drag_handler(
    State(state): State<DebugState>,
    Json(req): Json<DragRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<()>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Drag {
            from_selector: req.from_selector,
            to_selector: req.to_selector,
            steps: req.steps.unwrap_or(10),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<()>();
    }
    await_op(rx).await
}

async fn a11y_handler(
    State(state): State<DebugState>,
    Query(params): Query<A11yQueryParams>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<Vec<A11yNode>>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::A11y {
            selector: params.selector,
            depth: params.depth.unwrap_or(5),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<Vec<A11yNode>>();
    }
    await_op(rx).await
}

async fn batch_handler(
    State(state): State<DebugState>,
    Json(req): Json<BatchRequest>,
) -> impl IntoResponse {
    let mut results = Vec::with_capacity(req.operations.len());
    for (i, op) in req.operations.into_iter().enumerate() {
        let start = Instant::now();
        let name = match &op {
            BatchOperation::Screenshot { name, .. } => {
                name.clone().unwrap_or_else(|| format!("screenshot_{}", i))
            }
            _ => format!("op_{}", i),
        };
        let op_type = match &op {
            BatchOperation::Navigate { .. } => "navigate",
            BatchOperation::Screenshot { .. } => "screenshot",
            BatchOperation::Click { .. } => "click",
            BatchOperation::Evaluate { .. } => "evaluate",
            BatchOperation::Wait { .. } => "wait",
            BatchOperation::Scroll { .. } => "scroll",
            BatchOperation::Resize { .. } => "resize",
        }
        .to_string();

        let (success, data, error) = match execute_batch_op(&state, op).await {
            Ok(d) => (true, Some(d), None),
            Err(e) => (false, None, Some(e)),
        };
        results.push(BatchResult {
            name,
            op_type,
            success,
            data,
            error,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }
    ResponseJson(ApiResponse::ok(serde_json::json!({ "results": results })))
}

async fn execute_batch_op(
    state: &DebugState,
    op: BatchOperation,
) -> Result<serde_json::Value, String> {
    let br = state.browser.as_ref().ok_or("No browser")?;
    match op {
        BatchOperation::Navigate { url, wait_for } => {
            let target = if url.starts_with("http") {
                url
            } else {
                format!("{}{}", state.base_url, url)
            };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Navigate {
                url: target,
                wait_for,
                resp: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())?;
            r.map(|nav| serde_json::to_value(nav).unwrap_or_default())
        }
        BatchOperation::Screenshot {
            selector,
            full_page,
            ..
        } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Screenshot {
                selector,
                full_page: full_page.unwrap_or(false),
                resp: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())?;
            r.map(|ss| serde_json::json!({ "width": ss.width, "height": ss.height, "data_len": ss.data.len() }))
        }
        BatchOperation::Click { selector } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Click { selector, resp: tx })
                .await
                .map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())??;
            Ok(serde_json::json!({ "clicked": true }))
        }
        BatchOperation::Evaluate { expression } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Evaluate {
                expression,
                await_promise: false,
                resp: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())?;
            r.map(|ev| serde_json::json!({ "result": ev.result, "type": ev.r#type }))
        }
        BatchOperation::Wait { ms } => {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            Ok(serde_json::json!({ "waited_ms": ms }))
        }
        BatchOperation::Scroll {
            selector,
            direction,
            amount,
        } => {
            let (x, y) = match direction.as_deref() {
                Some("up") => (0.0, -(amount.unwrap_or(300.0))),
                Some("down") => (0.0, amount.unwrap_or(300.0)),
                Some("left") => (-(amount.unwrap_or(300.0)), 0.0),
                Some("right") => (amount.unwrap_or(300.0), 0.0),
                _ => (0.0, amount.unwrap_or(300.0)),
            };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Scroll {
                selector,
                x,
                y,
                resp: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())??;
            Ok(serde_json::json!({ "scrolled": true }))
        }
        BatchOperation::Resize {
            width,
            height,
            preset,
        } => {
            let (w, h) = match preset.as_deref() {
                Some("mobile") => (375, 812),
                Some("tablet") => (768, 1024),
                Some("desktop") => (1280, 720),
                Some("wide") => (1920, 1080),
                _ => (
                    width.unwrap_or(DEFAULT_VIEWPORT_W),
                    height.unwrap_or(DEFAULT_VIEWPORT_H),
                ),
            };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Resize {
                width: w,
                height: h,
                resp: tx,
            })
            .await
            .map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx)
                .await
                .map_err(|_| "timeout".to_string())?
                .map_err(|_| "channel closed".to_string())??;
            Ok(serde_json::json!({ "resized": [w, h] }))
        }
    }
}

async fn network_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<NetworkResponse>(),
    };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Network { resp: tx }).await.is_err() {
        return chan_closed::<NetworkResponse>();
    }
    await_op(rx).await
}

async fn performance_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<PerformanceMetrics>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::Performance { resp: tx })
        .await
        .is_err()
    {
        return chan_closed::<PerformanceMetrics>();
    }
    await_op(rx).await
}

async fn websocket_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<WebSocketInfo>(),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::WebSocket { resp: tx })
        .await
        .is_err()
    {
        return chan_closed::<WebSocketInfo>();
    }
    await_op(rx).await
}

async fn source_map_handler(Json(req): Json<SourceMapRequest>) -> impl IntoResponse {
    let frames = parse_wasm_stack(&req.stack);
    ResponseJson(ApiResponse::ok(SourceMapResponse {
        frames,
        raw: req.stack,
    }))
}

fn parse_wasm_stack(stack: &str) -> Vec<StackFrame> {
    let mut frames = Vec::new();
    for line in stack.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let raw = line.to_string();
        let (func, rest) = if let Some(at_pos) = line.find(" at ") {
            (Some(line[..at_pos].trim().to_string()), &line[at_pos + 4..])
        } else {
            (None, line)
        };
        let (file, line_num, col) = if let Some(paren_start) = rest.find('(') {
            let inner = if let Some(paren_end) = rest.rfind(')') {
                &rest[paren_start + 1..paren_end]
            } else {
                &rest[paren_start + 1..]
            };
            parse_location(inner)
        } else {
            parse_location(rest)
        };
        frames.push(StackFrame {
            file,
            line: line_num,
            col,
            func,
            raw,
        });
    }
    frames
}

fn parse_location(s: &str) -> (String, Option<u32>, Option<u32>) {
    let s = s.trim();
    if let Some(colon_pos) = s.rfind(':') {
        let after_colon = &s[colon_pos + 1..];
        if let Ok(col) = after_colon.parse::<u32>() {
            let before_col = &s[..colon_pos];
            if let Some(colon2) = before_col.rfind(':') {
                if let Ok(line) = before_col[colon2 + 1..].parse::<u32>() {
                    return (before_col[..colon2].to_string(), Some(line), Some(col));
                }
            }
            return (before_col.to_string(), None, Some(col));
        }
    }
    (s.to_string(), None, None)
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn svc_unavailable<T: Serialize>() -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::<T>::err("No browser available")),
    )
}
fn chan_closed<T: Serialize>() -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::<T>::err("Browser channel closed")),
    )
}

async fn await_op<T: Serialize>(
    rx: oneshot::Receiver<Result<T, String>>,
) -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    match tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await {
        Ok(Ok(Ok(d))) => (StatusCode::OK, ResponseJson(ApiResponse::ok(d))),
        Ok(Ok(Err(e))) => (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::err(e))),
        Ok(Err(_)) => chan_closed::<T>(),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            ResponseJson(ApiResponse::err("Operation timed out")),
        ),
    }
}
