use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{RwLock, mpsc, oneshot};

use axum::{
    Router,
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{delete, get, post},
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
    /// When true, include a default set of computed styles for the element.
    computed: Option<bool>,
    /// When true, describe every match (not just the first).
    all: Option<bool>,
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
    /// Populated when `all=true`: every matching element (not just the first).
    matches: Option<Vec<DomMatchEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomMatchEntry {
    tag: Option<String>,
    text: Option<String>,
    html: Option<String>,
    attributes: Option<serde_json::Map<String, serde_json::Value>>,
    visible: Option<bool>,
    rect: Option<RectResponse>,
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
    url: String,
    method: Option<String>,
    r#type: String,
    status: Option<u16>,
    duration: f64,
    size: f64,
    failed: Option<String>,
    /// CDP timestamp the request started (not serialized; used to order /network).
    #[serde(skip)]
    started: f64,
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

#[allow(dead_code)]
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
        /// Fetch a default set of computed styles for the element.
        computed: bool,
        /// Describe every match (not just the first) into `matches`.
        all: bool,
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
    Performance {
        resp: oneshot::Sender<Result<PerformanceMetrics, String>>,
    },
    NavigateHistory {
        /// true = back, false = forward
        back: bool,
        resp: oneshot::Sender<Result<(), String>>,
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
/// Anti-detection JS injected into every page before any site script runs.
/// Patches the most common headless-detection vectors.
#[cfg(feature = "debug-browser")]
const STEALTH_JS: &str = r#"
// Patch navigator.webdriver
Object.defineProperty(navigator, 'webdriver', { get: () => undefined });

// Restore window.chrome (headless removes it)
if (!window.chrome) {
    window.chrome = { runtime: {}, loadTimes: function(){}, csi: function(){} };
}

// Override permissions query for 'notifications' (headless returns 'denied')
const origQuery = window.navigator.permissions.query;
window.navigator.permissions.query = (parameters) =>
    parameters.name === 'notifications'
        ? Promise.resolve({ state: Notification.permission })
        : origQuery(parameters);

// Fake plugins array (headless has empty plugins)
Object.defineProperty(navigator, 'plugins', {
    get: () => [1, 2, 3, 4, 5],
});

// Fake languages (headless sometimes has empty languages)
Object.defineProperty(navigator, 'languages', {
    get: () => ['en-US', 'en'],
});
"#;

#[cfg(feature = "debug-browser")]
mod engine {
    use futures::{SinkExt, StreamExt};
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;
    use tokio::process::{Child, Command};
    use tokio::sync::{Mutex, RwLock, mpsc, oneshot};
    use tokio_tungstenite::tungstenite::Message;

    use super::*;

    const DEVTOOLS_POLL: Duration = Duration::from_millis(200);
    // Chrome in --single-process needs 8-10s for devtools, 30s for operations.
    const DEVTOOLS_TIMEOUT: Duration = Duration::from_secs(20);
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
            let raw = serde_json::to_string(&payload).map_err(|e| format!("cdp encode: {e}"))?;
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
        let secs = if raw > 4_000_000_000.0 {
            raw / 1000.0
        } else {
            raw
        };
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

    /// Same idea for the network/websocket capture maps (evict arbitrary
    /// entries once over the cap — order isn't load-bearing for the snapshot).
    fn cap_map<K: std::hash::Hash + Eq + Clone, V>(m: &mut HashMap<K, V>, cap: usize) {
        while m.len() > cap {
            let k = match m.keys().next().cloned() {
                Some(k) => k,
                None => break,
            };
            m.remove(&k);
        }
    }

    // ── launch + connect ─────────────────────────────────────────────────────

    pub(super) async fn spawn_browser(
        base_url: String,
        _initial_url: Option<String>,
        console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
        errors: Arc<RwLock<Vec<ErrorEntry>>>,
        network: Arc<RwLock<HashMap<String, NetworkResource>>>,
        websockets: Arc<RwLock<HashMap<String, WebSocketConn>>>,
        proxy: Option<String>,
    ) -> Result<BrowserHandle, String> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<BrowserCommand>(64);
        let connected = Arc::new(RwLock::new(false));

        let exe = resolve_executable_blocking().await?;
        let port = pick_free_port().ok_or_else(|| "no free port for devtools".to_string())?;

        let child: Child = {
            // Try normal multi-process mode first for best stability.
            // Single-process is a fallback for sandboxed envs where fork is blocked.
            let use_single_process = std::env::var("TAIRITSU_SINGLE_PROCESS").is_ok();
            let mut args = vec![
                "--headless=new".to_string(),
                "--no-sandbox".to_string(),
                "--disable-gpu".to_string(),
                "--no-first-run".to_string(),
                format!("--remote-debugging-port={port}"),
                format!("--window-size={DEFAULT_VIEWPORT_W},{DEFAULT_VIEWPORT_H}"),
            ];
            if use_single_process {
                args.push("--no-zygote".to_string());
                args.push("--single-process".to_string());
            }
            if let Some(ref p) = proxy {
                args.push(format!("--proxy-server={p}"));
            }
            args.push(base_url.clone());
            Command::new(&exe)
                .args(&args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true)
                .spawn()
                .map_err(|e| format!("failed to launch chrome ({exe}): {e}"))?
        };

        // Wait for the devtools HTTP endpoint, then read the browser ws URL.
        let ws_url = wait_for_devtools(port).await?;

        let (ws, _resp) = tokio_tungstenite::connect_async(&ws_url)
            .await
            .map_err(|e| format!("devtools ws connect failed: {e}"))?;
        tracing::info!("Debug browser connected (chromium raw-CDP)");

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
        let network_buf = network;
        let ws_buf = websockets;
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
                                    args.iter()
                                        .map(remote_object_text)
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                })
                                .unwrap_or_default();
                            let mut buf = console_buf.write().await;
                            buf.push(ConsoleEntry {
                                level,
                                text,
                                timestamp: cdp_ts(p.get("timestamp").and_then(|t| t.as_f64())),
                                source: Some("runtime".into()),
                            });
                            truncate_vec(&mut buf, 500);
                        }
                    }
                    "Runtime.exceptionThrown" => {
                        if let Some(ed) = v.get("params").and_then(|p| p.get("exceptionDetails")) {
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
                                        .map(|f| {
                                            let fn_ = f
                                                .get("functionName")
                                                .and_then(|x| x.as_str())
                                                .unwrap_or("<anon>");
                                            let url =
                                                f.get("url").and_then(|x| x.as_str()).unwrap_or("");
                                            let ln = f
                                                .get("lineNumber")
                                                .and_then(|x| x.as_i64())
                                                .unwrap_or(0);
                                            let co = f
                                                .get("columnNumber")
                                                .and_then(|x| x.as_i64())
                                                .unwrap_or(0);
                                            format!("    at {fn_} ({url}:{ln}:{co})")
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
                    // Network request lifecycle → /network buffer (real CDP
                    // capture, replacing the old performance-API polyfill).
                    "Network.requestWillBeSent" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let req = p.get("request");
                                let url = req
                                    .and_then(|r| r.get("url"))
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let method = req
                                    .and_then(|r| r.get("method"))
                                    .and_then(|x| x.as_str())
                                    .map(String::from);
                                let typ = p
                                    .get("type")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("Other")
                                    .to_string();
                                let started =
                                    p.get("timestamp").and_then(|t| t.as_f64()).unwrap_or(0.0);
                                let mut buf = network_buf.write().await;
                                buf.insert(
                                    id.to_string(),
                                    NetworkResource {
                                        name: url.clone(),
                                        url,
                                        method,
                                        r#type: typ,
                                        status: None,
                                        duration: 0.0,
                                        size: 0.0,
                                        failed: None,
                                        started,
                                    },
                                );
                                cap_map(&mut buf, 300);
                            }
                        }
                    }
                    "Network.responseReceived" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let mut buf = network_buf.write().await;
                                if let Some(entry) = buf.get_mut(id) {
                                    if let Some(resp) = p.get("response") {
                                        entry.status = resp
                                            .get("status")
                                            .and_then(|s| s.as_u64())
                                            .map(|s| s as u16);
                                        if let Some(mt) =
                                            resp.get("mimeType").and_then(|x| x.as_str())
                                        {
                                            entry.r#type = mt.to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    "Network.loadingFinished" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let finished_ts =
                                    p.get("timestamp").and_then(|t| t.as_f64()).unwrap_or(0.0);
                                let size = p
                                    .get("encodedDataLength")
                                    .and_then(|x| x.as_f64())
                                    .unwrap_or(0.0);
                                let mut buf = network_buf.write().await;
                                if let Some(entry) = buf.get_mut(id) {
                                    entry.size = size;
                                    entry.duration = (finished_ts - entry.started).max(0.0);
                                }
                            }
                        }
                    }
                    "Network.loadingFailed" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let err = p
                                    .get("errorText")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("failed")
                                    .to_string();
                                let mut buf = network_buf.write().await;
                                if let Some(entry) = buf.get_mut(id) {
                                    entry.failed = Some(err);
                                }
                            }
                        }
                    }
                    // WebSocket lifecycle → /websocket buffer (real CDP capture,
                    // replacing the old app-side _wsTracker polyfill).
                    "Network.webSocketCreated" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let url = p
                                    .get("url")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let ts = p.get("timestamp").and_then(|t| t.as_f64());
                                let mut buf = ws_buf.write().await;
                                buf.insert(
                                    id.to_string(),
                                    WebSocketConn {
                                        url,
                                        state: "connecting".into(),
                                        created_at_ms: ts,
                                    },
                                );
                                cap_map(&mut buf, 100);
                            }
                        }
                    }
                    "Network.webSocketHandshakeResponseReceived" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let mut buf = ws_buf.write().await;
                                if let Some(c) = buf.get_mut(id) {
                                    c.state = "open".into();
                                }
                            }
                        }
                    }
                    "Network.webSocketClosed" => {
                        if let Some(p) = v.get("params") {
                            if let Some(id) = p.get("requestId").and_then(|x| x.as_str()) {
                                let mut buf = ws_buf.write().await;
                                if let Some(c) = buf.get_mut(id) {
                                    c.state = "closed".into();
                                }
                            }
                        }
                    }
                    _ => {} // unknown event — deliberately ignored (skew-proof)
                }
            }
            *conn_reader.write().await = false;
            // Chrome's WebSocket closed — it crashed or was killed.
            // The dispatch loop holds the Child; when cmd_rx drops (all
            // BrowserHandles gone), kill_on_drop will reap it. But if
            // handles are still alive, we need to signal shutdown.
            tracing::warn!("Chrome WebSocket disconnected — browser lost");
        });

        // Enable the domains we use — with short timeout so we don't block
        // for 60s if Chrome is unresponsive in a constrained environment.
        let init_timeout = Duration::from_secs(5);
        let page_ok =
            tokio::time::timeout(init_timeout, client.command("Page.enable", json!({}))).await;
        if page_ok.is_err() {
            tracing::warn!("Page.enable timed out — Chrome may be unstable in this environment");
        }
        let _ =
            tokio::time::timeout(init_timeout, client.command("Runtime.enable", json!({}))).await;
        let _ =
            tokio::time::timeout(init_timeout, client.command("Network.enable", json!({}))).await;

        // Verify Chrome is actually responsive with a simple evaluate.
        let ping = tokio::time::timeout(Duration::from_secs(3), client.evaluate("1+1")).await;
        match ping {
            Ok(Ok(val)) if val.as_u64() == Some(2) => {
                tracing::info!("Chrome CDP verified responsive");
            }
            _ => {
                tracing::warn!("Chrome CDP not responsive — browser operations may fail");
                // Don't return error — let the server start; user can retry.
            }
        }

        // Anti-detection: inject stealth script before any page script runs.
        // This patches navigator.webdriver, window.chrome, permissions API,
        // plugins, and other telltale signs of headless automation.
        let _ = client
            .command(
                "Page.addScriptToEvaluateOnNewDocument",
                json!({
                    "source": STEALTH_JS
                }),
            )
            .await;

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
        crate::browser_fetch::resolve_executable()
            .map_err(|e| format!("no chrome/chromium could be resolved: {e}"))
    }

    /// Async wrapper: resolve() may perform a blocking HTTP download + zip
    /// extraction (runtime-fetch fallback), so it must NOT run on a tokio
    /// worker thread. Offload it to spawn_blocking.
    async fn resolve_executable_blocking() -> Result<String, String> {
        match tokio::task::spawn_blocking(resolve_executable).await {
            Ok(res) => res,
            Err(join_err) => Err(format!("browser resolver task failed: {join_err}")),
        }
    }

    // ── command dispatch ─────────────────────────────────────────────────────

    async fn dispatch_command(client: &CdpClient, cmd: BrowserCommand) {
        match cmd {
            BrowserCommand::Navigate {
                url,
                wait_for,
                resp,
            } => {
                let r = cmd_navigate(client, &url, wait_for.as_deref()).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Screenshot {
                selector,
                full_page,
                resp,
            } => {
                let r = cmd_screenshot(client, selector.as_deref(), full_page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Click { selector, resp } => {
                let r = cmd_click(client, &selector).await;
                let _ = resp.send(r);
            }
            BrowserCommand::TypeText {
                selector,
                text,
                clear_first,
                submit,
                resp,
            } => {
                let r = cmd_type(client, &selector, &text, clear_first, submit).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Evaluate {
                expression,
                await_promise,
                resp,
            } => {
                let r = cmd_evaluate(client, &expression, await_promise).await;
                let _ = resp.send(r);
            }
            BrowserCommand::DomQuery {
                selector,
                attribute,
                computed,
                all,
                resp,
            } => {
                let r = cmd_dom_query(client, &selector, attribute.as_deref(), computed, all).await;
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
            BrowserCommand::Scroll {
                selector,
                x,
                y,
                resp,
            } => {
                let r = cmd_scroll(client, selector.as_deref(), x, y).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Resize {
                width,
                height,
                resp,
            } => {
                let r = cmd_resize(client, width, height).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Viewport { resp } => {
                let r = cmd_viewport(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::A11y {
                selector,
                depth,
                resp,
            } => {
                let r = cmd_a11y(client, selector.as_deref(), depth).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Performance { resp } => {
                let r = cmd_performance(client).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Drag {
                from_selector,
                to_selector,
                steps,
                resp,
            } => {
                let r = cmd_drag(client, &from_selector, &to_selector, steps).await;
                let _ = resp.send(r);
            }
            BrowserCommand::NavigateHistory { back, resp } => {
                let r = cmd_navigate_history(client, back).await;
                let _ = resp.send(r);
            }
        }
    }

    /// Poll `document.readyState` until "complete" or `timeout_ms` elapses.
    /// Replaces fixed sleeps — faster on quick pages, robust against slow ones.
    /// Gives up (returns Ok) rather than failing the navigation on a timeout.
    async fn wait_ready_state(client: &CdpClient, timeout_ms: u64) -> Result<(), String> {
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        // Let the navigation kick in before the first read (avoids reading the
        // previous document's readyState immediately after Page.navigate).
        tokio::time::sleep(Duration::from_millis(50)).await;
        loop {
            let done = client
                .evaluate("document.readyState")
                .await
                .ok()
                .and_then(|v| v.as_str().map(|s| s == "complete"))
                .unwrap_or(false);
            if done || std::time::Instant::now() > deadline {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
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
            // Wait for the new document to finish loading, then poll the
            // tairitsu hydration marker (set after WASM hydrates the tree).
            let _ = wait_ready_state(client, 8_000).await;
            let deadline = std::time::Instant::now() + Duration::from_secs(10);
            loop {
                let hydrated = client
                    .evaluate("document.documentElement.dataset.tairitsuReady === 'hydrated'")
                    .await
                    .ok()
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if hydrated || std::time::Instant::now() > deadline {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } else {
            // Default / "load": just wait for the document to finish loading.
            let _ = wait_ready_state(client, 8_000).await;
        }
        let title = client
            .evaluate("document.title")
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();
        Ok(NavigateResponse {
            url: url.to_string(),
            title,
        })
    }

    /// Navigate the page history back (back=true) or forward (back=false) via
    /// `Page.getNavigationHistory` + `Page.navigateToHistoryEntry`. No-op (not
    /// an error) when already at the start/end of the history.
    async fn cmd_navigate_history(client: &CdpClient, back: bool) -> Result<(), String> {
        let resp = client
            .command("Page.getNavigationHistory", json!({}))
            .await
            .map_err(|e| format!("history: {e}"))?;
        let entries = resp
            .get("entries")
            .and_then(|e| e.as_array())
            .ok_or_else(|| "history: no entries".to_string())?;
        let curr = resp
            .get("currentIndex")
            .and_then(|i| i.as_u64())
            .ok_or_else(|| "history: no currentIndex".to_string())?;
        let target = if back {
            curr.checked_sub(1)
        } else {
            curr.checked_add(1)
        };
        let target = match target {
            Some(t) if (t as usize) < entries.len() => t,
            _ => return Ok(()), // at the start/end of history — nothing to do
        };
        let entry_id = entries[target as usize]
            .get("id")
            .and_then(|i| i.as_u64())
            .ok_or_else(|| "history: entry has no id".to_string())?;
        client
            .command(
                "Page.navigateToHistoryEntry",
                json!({ "entryId": entry_id }),
            )
            .await
            .map_err(|e| format!("history nav: {e}"))?;
        let _ = wait_ready_state(client, 8_000).await;
        Ok(())
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
            let raw = client
                .evaluate(&rect_js)
                .await
                .map_err(|e| format!("screenshot rect: {e}"))?;
            let s = raw
                .as_str()
                .ok_or_else(|| "screenshot rect: non-string".to_string())?;
            let rect: Value =
                serde_json::from_str(s).map_err(|e| format!("screenshot rect parse: {e}"))?;
            let clip = json!({
                "x": rect["x"], "y": rect["y"],
                "width": rect["width"], "height": rect["height"],
                "scale": rect["scale"],
            });
            let resp = client
                .command(
                    "Page.captureScreenshot",
                    json!({ "format": "png", "clip": clip }),
                )
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
        // Scroll the element into view and read its center in viewport
        // coordinates — that's where the real mouse events land.
        let rect_js = format!(
            r#"(() => {{ const el = document.querySelector({sel:?}); if (!el) throw 'element not found'; el.scrollIntoView({{ block: 'center', inline: 'center' }}); const r = el.getBoundingClientRect(); return JSON.stringify({{ x: r.x + r.width / 2, y: r.y + r.height / 2 }}); }})()"#,
            sel = selector,
        );
        let v = client
            .evaluate(&rect_js)
            .await
            .map_err(|e| format!("click: {e}"))?;
        let s = v
            .as_str()
            .ok_or_else(|| "click: rect not a string".to_string())?;
        let xy: Value = serde_json::from_str(s).map_err(|e| format!("click rect parse: {e}"))?;
        let x = xy
            .get("x")
            .and_then(|n| n.as_f64())
            .ok_or_else(|| "click: no x".to_string())?;
        let y = xy
            .get("y")
            .and_then(|n| n.as_f64())
            .ok_or_else(|| "click: no y".to_string())?;
        dispatch_mouse_click(client, x, y).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Dispatch a real left-button click (move → press → release) via CDP input
    /// events at viewport coords (x, y). Unlike a synthetic `el.click()`, this
    /// fires the full mousedown/mouseup/pointer sequence at real coordinates,
    /// so components driven by pointer events or hit-testing behave correctly.
    async fn dispatch_mouse_click(client: &CdpClient, x: f64, y: f64) -> Result<(), String> {
        client
            .command(
                "Input.dispatchMouseEvent",
                json!({ "type": "mouseMoved", "x": x, "y": y }),
            )
            .await
            .map_err(|e| format!("click move: {e}"))?;
        client
            .command(
                "Input.dispatchMouseEvent",
                json!({ "type": "mousePressed", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            )
            .await
            .map_err(|e| format!("click press: {e}"))?;
        client
            .command(
                "Input.dispatchMouseEvent",
                json!({ "type": "mouseReleased", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            )
            .await
            .map_err(|e| format!("click release: {e}"))?;
        Ok(())
    }

    async fn cmd_type(
        client: &CdpClient,
        selector: &str,
        text: &str,
        clear_first: bool,
        submit: bool,
    ) -> Result<(), String> {
        // Focus the field (and clear it if asked) via the DOM, then type the
        // text through the browser's real insertion path — fires input events
        // and handles unicode / IME / caret like a real user (previously this
        // stomped el.value directly, bypassing the keyboard entirely).
        let focus_js = format!(
            r#"(() => {{ const el = document.querySelector({sel:?}); if (!el) throw 'element not found'; el.focus(); if ({clear}) {{ el.value = ''; el.dispatchEvent(new Event('input', {{ bubbles: true }})); }} }})()"#,
            sel = selector,
            clear = clear_first,
        );
        client
            .evaluate(&focus_js)
            .await
            .map_err(|e| format!("type focus: {e}"))?;
        if !text.is_empty() {
            client
                .command("Input.insertText", json!({ "text": text }))
                .await
                .map_err(|e| format!("type: {e}"))?;
        }
        if submit {
            // Press Enter to submit the surrounding form.
            for ty in ["keyDown", "keyUp"] {
                client
                    .command(
                        "Input.dispatchKeyEvent",
                        json!({ "type": ty, "key": "Enter", "code": "Enter", "windowsVirtualKeyCode": 13 }),
                    )
                    .await
                    .map_err(|e| format!("type submit ({ty}): {e}"))?;
            }
        }
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
        Ok(EvaluateResponse {
            result: val,
            r#type: type_name.into(),
        })
    }

    async fn cmd_dom_query(
        client: &CdpClient,
        selector: &str,
        attribute: Option<&str>,
        computed: bool,
        all: bool,
    ) -> Result<DomNodeResponse, String> {
        if let Some(attr) = attribute {
            let js = format!(
                "(() => {{ const el = document.querySelector({sel:?}); if (!el) return null; return el.getAttribute({attr:?}); }})()",
                sel = selector,
                attr = attr,
            );
            let val = client
                .evaluate(&js)
                .await
                .map_err(|e| format!("dom query: {e}"))?;
            let r = val.as_str().map(|s| s.to_string());
            let count = if r.is_some() { 1 } else { 0 };
            return Ok(DomNodeResponse {
                tag: None,
                text: r,
                html: None,
                attributes: None,
                visible: None,
                count,
                rect: None,
                computed: None,
                matches: None,
            });
        }
        // Describe every match (list query) — returns {count, matches:[…]}.
        if all {
            let js_all = r#"
(() => {
  const els = document.querySelectorAll(__SEL__);
  const matches = Array.from(els).map(el => {
    const r = el.getBoundingClientRect();
    return { tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 500), html: el.outerHTML.substring(0, 2000), attributes: Object.fromEntries(Array.from(el.attributes).map(a => [a.name, a.value])), visible: r.width > 0 && r.height > 0, rect: { x: r.x, y: r.y, width: r.width, height: r.height } };
  });
  if (!matches.length) throw 'not found';
  return JSON.stringify({ count: matches.length, matches: matches });
})()
"#.replace("__SEL__", &format!("{selector:?}"));
            let val = client
                .evaluate(&js_all)
                .await
                .map_err(|e| format!("dom query: {e}"))?;
            let json_str = val
                .as_str()
                .ok_or_else(|| "dom query: non-string result".to_string())?;
            return serde_json::from_str::<DomNodeResponse>(json_str)
                .map_err(|e| format!("dom query deserialize: {e}"));
        }
        // Full element description. `attributes` (not `attrs`) so it maps to the
        // DomNodeResponse field; `computed` fetches a default property set when
        // requested (folding the old /dom/computed endpoint into /dom).
        let js_body = r#"
(() => {
  const els = document.querySelectorAll(__SEL__);
  if (!els.length) throw 'not found';
  const el = els[0];
  const r = el.getBoundingClientRect();
  const attributes = Object.fromEntries(Array.from(el.attributes).map(a => [a.name, a.value]));
  var computed = null;
  if (__COMPUTED__) {
    const cs = getComputedStyle(el);
    const props = ['display','visibility','opacity','color','background-color','width','height','margin-top','margin-right','margin-bottom','margin-left','padding-top','padding-right','padding-bottom','padding-left','border-width','border-radius','font-size','font-weight','line-height','position','z-index','overflow','cursor'];
    computed = Object.fromEntries(props.map(p => [p, cs.getPropertyValue(p)]));
  }
  return JSON.stringify({ tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 2000), html: el.outerHTML.substring(0, 5000), attributes: attributes, visible: r.width > 0 && r.height > 0, count: els.length, rect: { x: r.x, y: r.y, width: r.width, height: r.height }, computed: computed });
})()
"#.replace("__SEL__", &format!("{selector:?}"))
   .replace("__COMPUTED__", &computed.to_string());
        let val = client
            .evaluate(&js_body)
            .await
            .map_err(|e| format!("dom query: {e}"))?;
        let json_str = val
            .as_str()
            .ok_or_else(|| "dom query: non-string result".to_string())?;
        serde_json::from_str::<DomNodeResponse>(json_str)
            .map_err(|e| format!("dom query deserialize: {e}"))
    }

    async fn cmd_is_ready(client: &CdpClient) -> Result<ReadyResponse, String> {
        let js = r#"(() => { const w = !!globalThis.__wasmExports; const h = document.documentElement.dataset.tairitsuReady === 'hydrated'; return JSON.stringify({ ready: w && h, wasm_loaded: w, hydrated: h, url: location.href }); })()"#;
        let val = client
            .evaluate(js)
            .await
            .map_err(|e| format!("is_ready: {e}"))?;
        let json_str = val
            .as_str()
            .ok_or_else(|| "is_ready: non-string".to_string())?;
        serde_json::from_str::<ReadyResponse>(json_str)
            .map_err(|e| format!("is_ready deserialize: {e}"))
    }

    async fn cmd_press(client: &CdpClient, key: &str) -> Result<(), String> {
        let k = resolve_key(key)?;
        // A bare printable char (no modifiers) is inserted via the real text
        // path; everything else (special keys, modifier combos) goes through
        // raw keyDown/keyUp dispatch. The keyDown carries `text` ("\r" for
        // Enter, "\t" for Tab, …) so the browser performs its default action
        // (form-submit, focus-traverse). All real CDP input — not the
        // synthetic JS KeyboardEvent this used to dispatch on `document`.
        if k.printable && k.modifiers == 0 {
            client
                .command("Input.insertText", json!({ "text": k.text }))
                .await
                .map_err(|e| format!("press: {e}"))?;
        } else {
            // type "keyDown" (not "rawKeyDown") when there's text, matching
            // how Playwright dispatches keys that carry a text payload — this
            // is what lets default actions (form submit on Enter, focus
            // traversal on Tab) actually fire.
            let key_type = if k.text.is_empty() {
                "rawKeyDown"
            } else {
                "keyDown"
            };
            let mut down = json!({
                "type": key_type, "key": k.key, "code": k.code, "modifiers": k.modifiers
            });
            if k.vk != 0 {
                down["windowsVirtualKeyCode"] = json!(k.vk);
            }
            if !k.text.is_empty() {
                down["text"] = json!(k.text);
            }
            client
                .command("Input.dispatchKeyEvent", down)
                .await
                .map_err(|e| format!("press down: {e}"))?;
            let mut up = json!({
                "type": "keyUp", "key": k.key, "code": k.code, "modifiers": k.modifiers
            });
            if k.vk != 0 {
                up["windowsVirtualKeyCode"] = json!(k.vk);
            }
            client
                .command("Input.dispatchKeyEvent", up)
                .await
                .map_err(|e| format!("press up: {e}"))?;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    struct KeySpec {
        modifiers: u8,
        key: String,
        code: String,
        vk: u32,
        /// `text` to carry on the keyDown (e.g. "\r" for Enter, "\t" for Tab,
        /// the char itself for printable keys). Drives the browser's default
        /// actions (form-submit on Enter, focus traversal on Tab, …).
        text: String,
        /// A bare printable character with no modifiers — type it via
        /// `Input.insertText` rather than keyDown/keyUp.
        printable: bool,
    }

    /// Resolve a press-key spec (e.g. "Enter", "ArrowUp", "Ctrl+a", "F2") into
    /// the CDP input fields. Modifier prefixes (Alt/Ctrl/Control/Meta/Command/
    /// Cmd/Shift) are parsed off; the remainder is matched against the named
    /// keys (Enter, Tab, arrows, F1-F12, …) or treated as a single printable
    /// character.
    fn resolve_key(spec: &str) -> Result<KeySpec, String> {
        let mut modifiers: u8 = 0;
        let mut name = String::new();
        for part in spec.split('+') {
            match part.trim() {
                "Alt" | "AltGraph" => modifiers |= 1,
                "Control" | "Ctrl" => modifiers |= 2,
                "Meta" | "Command" | "Cmd" => modifiers |= 4,
                "Shift" => modifiers |= 8,
                other => {
                    if !name.is_empty() {
                        name.push('+');
                    }
                    name.push_str(other);
                }
            }
        }
        if name.is_empty() {
            return Err(format!("empty key spec: {spec}"));
        }
        // (key, code, vk, text, printable)
        let (key, code, vk, text, printable): (String, String, u32, String, bool) =
            match name.as_str() {
                "Enter" | "Return" => ("Enter".into(), "Enter".into(), 13, "\r".into(), false),
                "Tab" => ("Tab".into(), "Tab".into(), 9, "\t".into(), false),
                "Escape" | "Esc" => ("Escape".into(), "Escape".into(), 27, String::new(), false),
                "Backspace" => (
                    "Backspace".into(),
                    "Backspace".into(),
                    8,
                    String::new(),
                    false,
                ),
                "Delete" => ("Delete".into(), "Delete".into(), 46, String::new(), false),
                "ArrowUp" => ("ArrowUp".into(), "ArrowUp".into(), 38, String::new(), false),
                "ArrowDown" => (
                    "ArrowDown".into(),
                    "ArrowDown".into(),
                    40,
                    String::new(),
                    false,
                ),
                "ArrowLeft" => (
                    "ArrowLeft".into(),
                    "ArrowLeft".into(),
                    37,
                    String::new(),
                    false,
                ),
                "ArrowRight" => (
                    "ArrowRight".into(),
                    "ArrowRight".into(),
                    39,
                    String::new(),
                    false,
                ),
                "Home" => ("Home".into(), "Home".into(), 36, String::new(), false),
                "End" => ("End".into(), "End".into(), 35, String::new(), false),
                "PageUp" => ("PageUp".into(), "PageUp".into(), 33, String::new(), false),
                "PageDown" => (
                    "PageDown".into(),
                    "PageDown".into(),
                    34,
                    String::new(),
                    false,
                ),
                "Space" => (" ".into(), "Space".into(), 32, " ".into(), false),
                f if f.starts_with('F') && f.len() >= 2 => {
                    let n: u32 = f[1..]
                        .parse()
                        .map_err(|_| format!("bad function key: {f}"))?;
                    if !(1..=12).contains(&n) {
                        return Err(format!("unsupported function key: {f}"));
                    }
                    (f.to_string(), f.to_string(), 111 + n, String::new(), false)
                }
                c if c.chars().count() == 1 => {
                    let ch = c.chars().next().unwrap();
                    let code = match ch {
                        'a'..='z' => format!("Key{}", ch.to_ascii_uppercase()),
                        'A'..='Z' => format!("Key{ch}"),
                        '0'..='9' => format!("Digit{ch}"),
                        _ => String::new(),
                    };
                    (c.to_string(), code, 0, c.to_string(), true)
                }
                other => return Err(format!("unknown key: {other}")),
            };
        Ok(KeySpec {
            modifiers,
            key,
            code,
            vk,
            text,
            printable,
        })
    }

    async fn cmd_scroll(
        client: &CdpClient,
        selector: Option<&str>,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
        let js = if let Some(sel) = selector {
            format!(
                r#"(() => {{ const el = document.querySelector({sel:?}); if (el) el.scrollBy({x}, {y}); }})()"#
            )
        } else {
            format!(r#"window.scrollBy({x}, {y})"#)
        };
        client
            .evaluate(&js)
            .await
            .map_err(|e| format!("scroll: {e}"))?;
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
        let val = client
            .evaluate(js)
            .await
            .map_err(|e| format!("viewport: {e}"))?;
        let json_str = val
            .as_str()
            .ok_or_else(|| "viewport: non-string".to_string())?;
        serde_json::from_str::<ViewportResponse>(json_str)
            .map_err(|e| format!("viewport deserialize: {e}"))
    }

    async fn cmd_a11y(
        client: &CdpClient,
        selector: Option<&str>,
        depth: u32,
    ) -> Result<Vec<A11yNode>, String> {
        // Selector-scoped snapshots keep the JS walker (it scopes to an element
        // easily). Whole-page snapshots prefer the browser's real accessibility
        // tree — accurate computed names/roles, shadow-DOM aware — and fall
        // back to the JS walker if the CDP tree is unavailable or empty.
        if selector.is_some() {
            return a11y_via_js(client, selector, depth).await;
        }
        match a11y_via_cdp(client, depth).await {
            Ok(nodes) if !nodes.is_empty() => Ok(nodes),
            _ => a11y_via_js(client, selector, depth).await,
        }
    }

    /// Whole-page accessibility tree from the browser's real a11y domain — more
    /// accurate than the JS walker (computed names, implicit/ARIA roles, shadow
    /// DOM). Ignored nodes are hoisted so structural wrappers don't consume the
    /// caller's depth budget.
    async fn a11y_via_cdp(client: &CdpClient, depth: u32) -> Result<Vec<A11yNode>, String> {
        let resp = client
            .command("Accessibility.getFullAXTree", json!({}))
            .await
            .map_err(|e| format!("a11y cdp: {e}"))?;
        let nodes = resp
            .get("nodes")
            .and_then(|n| n.as_array())
            .cloned()
            .unwrap_or_default();
        if nodes.is_empty() {
            return Ok(vec![]);
        }
        let by_id: HashMap<String, Value> = nodes
            .iter()
            .filter_map(|n| {
                n.get("nodeId")
                    .and_then(|v| v.as_str())
                    .map(|id| (id.to_string(), n.clone()))
            })
            .collect();
        // Roots: nodes with no parent (or a parent absent from the tree).
        let mut root_ids: Vec<String> = nodes
            .iter()
            .filter_map(|n| {
                let id = n.get("nodeId").and_then(|v| v.as_str())?;
                let pid = n.get("parentId").and_then(|v| v.as_str());
                match pid {
                    None => Some(id.to_string()),
                    Some(p) if !by_id.contains_key(p) => Some(id.to_string()),
                    _ => None,
                }
            })
            .collect();
        if root_ids.is_empty() {
            if let Some(id) = nodes
                .first()
                .and_then(|n| n.get("nodeId"))
                .and_then(|v| v.as_str())
            {
                root_ids.push(id.to_string());
            }
        }
        Ok(root_ids
            .iter()
            .flat_map(|id| a11y_collect(id, &by_id, 0, depth))
            .collect())
    }

    fn a11y_collect(
        id: &str,
        by_id: &HashMap<String, Value>,
        depth: u32,
        max_depth: u32,
    ) -> Vec<A11yNode> {
        let n = match by_id.get(id) {
            Some(n) => n,
            None => return vec![],
        };
        let child_ids: Vec<String> = n
            .get("childIds")
            .and_then(|c| c.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        // Ignored nodes aren't exposed to assistive tech — surface their
        // children at the same depth instead of emitting the wrapper.
        if n.get("ignored").and_then(|v| v.as_bool()).unwrap_or(false) {
            return child_ids
                .iter()
                .flat_map(|c| a11y_collect(c, by_id, depth, max_depth))
                .collect();
        }
        if depth > max_depth {
            return vec![];
        }
        let children: Vec<A11yNode> = if depth < max_depth {
            child_ids
                .iter()
                .flat_map(|c| a11y_collect(c, by_id, depth + 1, max_depth))
                .collect()
        } else {
            vec![]
        };
        vec![A11yNode {
            name: a11y_str(n, "name"),
            role: a11y_str(n, "role"),
            description: a11y_str(n, "description"),
            states: a11y_states(n),
            tag: None,
            children,
        }]
    }

    fn a11y_str(n: &Value, field: &str) -> Option<String> {
        n.get(field)
            .and_then(|o| o.get("value"))
            .and_then(|v| v.as_str())
            .map(String::from)
    }

    fn a11y_states(n: &Value) -> Vec<String> {
        let mut out = Vec::new();
        let Some(props) = n.get("properties").and_then(|p| p.as_array()) else {
            return out;
        };
        for p in props {
            let name = p.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let val = p.get("value").and_then(|v| v.get("value"));
            match name {
                "disabled" if val.and_then(|v| v.as_bool()).unwrap_or(false) => {
                    out.push("disabled".into())
                }
                "expanded" => match val.and_then(|v| v.as_bool()) {
                    Some(true) => out.push("expanded".into()),
                    Some(false) => out.push("collapsed".into()),
                    _ => {}
                },
                "checked" => match val {
                    Some(v) if v.as_bool() == Some(true) => out.push("checked".into()),
                    Some(v) if v.as_str() == Some("mixed") => out.push("mixed".into()),
                    _ => {}
                },
                "selected" if val.and_then(|v| v.as_bool()).unwrap_or(false) => {
                    out.push("selected".into())
                }
                "focused" if val.and_then(|v| v.as_bool()).unwrap_or(false) => {
                    out.push("focused".into())
                }
                "hidden" if val.and_then(|v| v.as_bool()).unwrap_or(false) => {
                    out.push("hidden".into())
                }
                _ => {}
            }
        }
        out
    }

    async fn a11y_via_js(
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
        let val = client
            .evaluate(&js_body)
            .await
            .map_err(|e| format!("a11y: {e}"))?;
        let json_str = val.as_str().ok_or_else(|| "a11y: non-string".to_string())?;
        serde_json::from_str::<Vec<A11yNode>>(json_str)
            .map_err(|e| format!("a11y deserialize: {e}"))
    }

    async fn cmd_performance(client: &CdpClient) -> Result<PerformanceMetrics, String> {
        let js = r#"(() => { var nav = performance.getEntriesByType('navigation')[0] || {}; var fcp = null; try { fcp = performance.getEntriesByName('first-contentful-paint')[0].startTime || null; } catch(e) {} var dn = document.querySelectorAll('*').length; var heap = null; try { heap = Math.round((performance.memory ? performance.memory.usedJSHeapSize : 0) / 1048576 * 100) / 100; } catch(e) {} return JSON.stringify({ dom_content_loaded_ms: Math.round((nav.domContentLoadedEventEnd - nav.startTime) * 100) / 100 || null, dom_complete_ms: Math.round((nav.domComplete - nav.startTime) * 100) / 100 || null, load_event_ms: Math.round((nav.loadEventEnd - nav.startTime) * 100) / 100 || null, fcp_ms: fcp ? Math.round(fcp * 100) / 100 : null, lcp_ms: null, cls: null, dom_nodes: dn, js_heap_used_mb: heap, wasm_loaded: !!globalThis.__wasmExports, hydrated: document.documentElement.dataset.tairitsuReady === 'hydrated', timestamp: new Date().toISOString() }); })()"#;
        let val = client
            .evaluate(js)
            .await
            .map_err(|e| format!("performance: {e}"))?;
        let json_str = val
            .as_str()
            .ok_or_else(|| "performance: non-string".to_string())?;
        serde_json::from_str::<PerformanceMetrics>(json_str)
            .map_err(|e| format!("performance deserialize: {e}"))
    }

    async fn cmd_drag(
        client: &CdpClient,
        from_selector: &str,
        to_selector: &str,
        steps: u32,
    ) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ var src = document.querySelector({from:?}); var dst = document.querySelector({to:?}); if (!src || !dst) throw 'element not found'; var sr = src.getBoundingClientRect(); var dr = dst.getBoundingClientRect(); var sx = sr.x + sr.width/2, sy = sr.y + sr.height/2; var dx = dr.x + dr.width/2, dy = dr.y + dr.height/2; src.dispatchEvent(new MouseEvent('mousedown', {{clientX: sx, clientY: sy, bubbles: true}})); for (var i = 1; i <= {steps}; i++) {{ var t = i/{steps}; var cx = sx + (dx - sx)*t, cy = sy + (dy - sy)*t; document.dispatchEvent(new MouseEvent('mousemove', {{clientX: cx, clientY: cy, bubbles: true}})); }} dst.dispatchEvent(new MouseEvent('mouseup', {{clientX: dx, clientY: dy, bubbles: true}})); dst.dispatchEvent(new MouseEvent('drop', {{clientX: dx, clientY: dy, bubbles: true}})); }})()"#,
            from = from_selector,
            to = to_selector,
            steps = steps,
        );
        client
            .evaluate(&js)
            .await
            .map_err(|e| format!("drag: {e}"))?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
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
    network: Arc<RwLock<HashMap<String, NetworkResource>>>,
    websockets: Arc<RwLock<HashMap<String, WebSocketConn>>>,
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
    /// Optional proxy server for Chrome (e.g. "http://localhost:7890").
    pub proxy: Option<String>,
}

pub async fn start_debug_server(cfg: DebugServerConfig, debug_port: u16) -> anyhow::Result<()> {
    let base_url = cfg.base_url.clone();
    let dev_port = cfg.dev_port;
    let console_log = Arc::new(RwLock::new(Vec::new()));
    let errors = Arc::new(RwLock::new(Vec::new()));
    let network = Arc::new(RwLock::new(HashMap::new()));
    let websockets = Arc::new(RwLock::new(HashMap::new()));

    #[cfg(feature = "debug-browser")]
    let (browser, browser_engine) = {
        tracing::info!("Debug browser engine: chromium (headless CDP)");
        match tokio::time::timeout(
            Duration::from_secs(45),
            engine::spawn_browser(
                base_url.clone(),
                None,
                console_log.clone(),
                errors.clone(),
                network.clone(),
                websockets.clone(),
                cfg.proxy.clone(),
            ),
        )
        .await
        {
            Ok(Ok(b)) => (Some(Arc::new(b)), "chromium".to_string()),
            Ok(Err(e)) => {
                tracing::error!("[debug-browser] Failed: {e}");
                (None, "none".to_string())
            }
            Err(_) => {
                tracing::error!("[debug-browser] Timed out after 30s");
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
        network,
        websockets,
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
        .route("/back", post(back_handler))
        .route("/forward", post(forward_handler))
        .route("/screenshot", post(screenshot_handler))
        .route("/click", post(click_handler))
        .route("/type", post(type_handler))
        .route("/press", post(press_handler))
        .route("/scroll", post(scroll_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/wait-for-selector", post(wait_for_selector_handler))
        .route("/console", get(console_handler))
        .route("/console", delete(console_clear_handler))
        .route("/dom", get(dom_query_handler))
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

    tracing::info!(
        "Debug API v{} listening on http://localhost:{}",
        DEBUG_API_VERSION,
        debug_port
    );
    tracing::info!(
        "Endpoints: /health /info /ready /navigate /back /forward /screenshot /click /type /press /scroll /evaluate /console /dom /viewport /resize /errors /drag /a11y /batch /network /performance /websocket /source-map"
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── HTTP handlers ─────────────────────────────────────────────────────────

async fn health_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        version: "0.5.13".into(),
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
        version: "0.5.13".into(),
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
    // Treat absolute schemes as-is; only relative paths get resolved against
    // the app's dev-server base_url.
    let target = if req.url.starts_with("http:")
        || req.url.starts_with("https:")
        || req.url.starts_with("data:")
        || req.url.starts_with("about:")
        || req.url.starts_with("blob:")
        || req.url.starts_with("file:")
    {
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

async fn back_handler(State(state): State<DebugState>) -> impl IntoResponse {
    history_step(&state, true).await
}

async fn forward_handler(State(state): State<DebugState>) -> impl IntoResponse {
    history_step(&state, false).await
}

async fn history_step(
    state: &DebugState,
    back: bool,
) -> ResponseJson<ApiResponse<serde_json::Value>> {
    let br = match &state.browser {
        Some(b) => b,
        None => return ResponseJson(ApiResponse::err("browser not connected")),
    };
    let (tx, rx) = oneshot::channel();
    if br
        .send(BrowserCommand::NavigateHistory { back, resp: tx })
        .await
        .is_err()
    {
        return ResponseJson(ApiResponse::err("browser channel closed"));
    }
    match rx.await {
        Ok(Ok(())) => ResponseJson(ApiResponse::ok(serde_json::Value::Null)),
        Ok(Err(e)) => ResponseJson(ApiResponse::err(e)),
        Err(_) => ResponseJson(ApiResponse::err("browser channel closed")),
    }
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

/// Wait for a CSS selector to appear in the DOM. Polls every 200ms up to a
/// configurable timeout. Returns the element count found.
#[derive(serde::Deserialize)]
struct WaitForSelectorRequest {
    selector: String,
    #[serde(default = "default_wait_timeout")]
    timeout_ms: u64,
}

fn default_wait_timeout() -> u64 {
    10_000
}

#[derive(serde::Serialize)]
struct WaitForSelectorResponse {
    selector: String,
    found: bool,
    count: usize,
    elapsed_ms: u64,
}

async fn wait_for_selector_handler(
    State(state): State<DebugState>,
    Json(req): Json<WaitForSelectorRequest>,
) -> impl IntoResponse {
    let br = match &state.browser {
        Some(b) => b,
        None => return svc_unavailable::<WaitForSelectorResponse>(),
    };

    let start = std::time::Instant::now();
    let deadline = start + Duration::from_millis(req.timeout_ms);
    let check_js = format!(
        "document.querySelectorAll({}).length",
        serde_json::to_string(&req.selector).unwrap_or_else(|_| "''".into())
    );

    let result: Result<WaitForSelectorResponse, String> = loop {
        let (tx, rx) = oneshot::channel();
        if br
            .send(BrowserCommand::Evaluate {
                expression: check_js.clone(),
                await_promise: false,
                resp: tx,
            })
            .await
            .is_err()
        {
            return chan_closed::<WaitForSelectorResponse>();
        }

        let (status, json) = await_op::<EvaluateResponse>(rx).await;
        if status.is_success() {
            if let Some(ref data) = json.0.data {
                let count = data.result.as_u64().unwrap_or(0) as usize;
                if count > 0 {
                    break Ok(WaitForSelectorResponse {
                        selector: req.selector.clone(),
                        found: true,
                        count,
                        elapsed_ms: start.elapsed().as_millis() as u64,
                    });
                }
            }
        }

        if std::time::Instant::now() >= deadline {
            break Ok(WaitForSelectorResponse {
                selector: req.selector.clone(),
                found: false,
                count: 0,
                elapsed_ms: start.elapsed().as_millis() as u64,
            });
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    };

    match result {
        Ok(data) => (StatusCode::OK, ResponseJson(ApiResponse::ok(data))),
        Err(e) => (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::err(e))),
    }
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
            computed: params.computed.unwrap_or(false),
            all: params.all.unwrap_or(false),
            resp: tx,
        })
        .await
        .is_err()
    {
        return chan_closed::<DomNodeResponse>();
    }
    await_op(rx).await
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
    let map = state.network.read().await;
    let mut resources: Vec<NetworkResource> = map.values().cloned().collect();
    resources.sort_by(|a, b| {
        a.started
            .partial_cmp(&b.started)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ResponseJson(ApiResponse::ok(NetworkResponse { resources }))
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
    let map = state.websockets.read().await;
    let connections: Vec<WebSocketConn> = map.values().cloned().collect();
    let active_count = connections
        .iter()
        .filter(|c| c.state == "open" || c.state == "connecting")
        .count() as u32;
    ResponseJson(ApiResponse::ok(WebSocketInfo {
        active_count,
        connections,
    }))
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
