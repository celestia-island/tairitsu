//! Debug/inspection API server for agent-driven browser automation.
//!
//! When `--debug` is passed to `tairitsu dev`, this module spawns an Axum
//! server on a separate port (default: dev-port + 1) that exposes endpoints
//! for screenshots, DOM queries, click/input simulation, and JS evaluation.
//!
//! With the `debug-browser` feature, a headless Chromium is launched and
//! controlled via CDP (Chrome DevTools Protocol). Without it, browser-
//! dependent endpoints return 503.
//!
//! Agents connect via HTTP and follow the protocol documented in
//! `docs/en/skills/debug-agent.md`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;

const DEBUG_API_VERSION: &str = "0.1.0";
const DEFAULT_VIEWPORT_W: u32 = 1280;
const DEFAULT_VIEWPORT_H: u32 = 720;
const OP_TIMEOUT_SECS: u64 = 30;

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Self { Self { ok: true, data: Some(data), error: None } }
    fn err(msg: impl Into<String>) -> Self { Self { ok: false, data: None, error: Some(msg.into()) } }
}

#[derive(Debug, Clone, Serialize, Deserialize)] struct HealthResponse {
    status: String, version: String, api_version: String, uptime_secs: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct InfoResponse {
    version: String, api_version: String, dev_port: u16, debug_port: u16,
    dist_dir: String, package_name: String, pid: u32,
    started_at_iso: String, uptime_secs: u64,
    browser_connected: bool, browser_engine: String,
}
#[derive(Debug, Clone, Deserialize)] struct NavigateRequest {
    url: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct NavigateResponse {
    url: String, title: String,
}
#[derive(Debug, Clone, Deserialize, Default)] struct ScreenshotParams {
    selector: Option<String>, full_page: Option<bool>, format: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct ScreenshotResponse {
    data: String, mime_type: String, width: u32, height: u32,
}
#[derive(Debug, Clone, Deserialize)] struct ClickRequest {
    selector: String, button: Option<String>, modifiers: Option<Vec<String>>,
}
#[derive(Debug, Clone, Deserialize)] struct TypeRequest {
    selector: String, text: String, clear_first: Option<bool>, submit: Option<bool>,
}
#[derive(Debug, Clone, Deserialize)] struct EvaluateRequest {
    expression: String, await_promise: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct EvaluateResponse {
    result: serde_json::Value, r#type: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct ConsoleEntry {
    level: String, text: String, timestamp: String, source: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct ConsoleResponse {
    entries: Vec<ConsoleEntry>,
}
#[derive(Debug, Clone, Deserialize, Default)] struct DomQueryParams {
    selector: String, attribute: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)] struct DomNodeResponse {
    tag: Option<String>, text: Option<String>, html: Option<String>,
    attributes: Option<serde_json::Map<String, serde_json::Value>>,
    visible: Option<bool>, count: usize,
}

// ── Browser command channel ───────────────────────────────────────────────

enum BrowserCommand {
    Navigate { url: String, resp: oneshot::Sender<Result<NavigateResponse, String>> },
    Screenshot { selector: Option<String>, full_page: bool, resp: oneshot::Sender<Result<ScreenshotResponse, String>> },
    Click { selector: String, resp: oneshot::Sender<Result<(), String>> },
    TypeText { selector: String, text: String, clear_first: bool, submit: bool, resp: oneshot::Sender<Result<(), String>> },
    Evaluate { expression: String, await_promise: bool, resp: oneshot::Sender<Result<EvaluateResponse, String>> },
    DomQuery { selector: String, attribute: Option<String>, resp: oneshot::Sender<Result<DomNodeResponse, String>> },
    Shutdown,
}

// ── BrowserHandle ─────────────────────────────────────────────────────────

struct BrowserHandle {
    tx: mpsc::Sender<BrowserCommand>,
    connected: Arc<RwLock<bool>>,
}

impl BrowserHandle {
    async fn send(&self, cmd: BrowserCommand) -> Result<(), String> {
        self.tx.send(cmd).await.map_err(|e| e.to_string())
    }
    async fn is_connected(&self) -> bool { *self.connected.read().await }
}

// ── CDP-based Browser Engine ─────────────────────────────────────────────

#[cfg(feature = "debug-browser")]
mod engine {
    use super::*;
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    use tokio_tungstenite::connect_async;

    /// Find a chromium binary on the system.
    fn find_chromium() -> Option<String> {
        let candidates = [
            "chromium-browser", "chromium", "google-chrome",
            "google-chrome-stable", "chrome", "microsoft-edge",
        ];
        if let Ok(path) = std::env::var("PATH") {
            for dir in path.split(':') {
                for name in &candidates {
                    let candidate = std::path::Path::new(dir).join(name);
                    if candidate.is_file() {
                        return Some(candidate.to_string_lossy().into_owned());
                    }
                }
            }
        }
        let extra = [
            "/usr/bin/google-chrome", "/usr/bin/chromium-browser",
            "/snap/bin/chromium", "/usr/bin/microsoft-edge-stable",
        ];
        for p in &extra {
            if std::path::Path::new(p).exists() {
                return Some(p.to_string());
            }
        }
        None
    }

    pub(super) fn spawn_browser(
        base_url: String,
        _initial_url: Option<String>,
        _console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    ) -> Result<BrowserHandle, String> {
        let chrome = find_chromium().ok_or_else(|| -> String {
            "No chromium/chrome binary found for debug browser".into()
        })?;

        crate::log_info!("Debug browser engine: {} (CDP+headless)", chrome);

        let (cmd_tx, cmd_rx) = mpsc::channel::<BrowserCommand>(64);
        let connected = Arc::new(RwLock::new(false));
        let conn = connected.clone();

        std::thread::Builder::new()
            .name("tairitsu-debug-cdp".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("browser runtime");
                rt.block_on(run_cdp_loop(chrome, base_url, cmd_rx, conn));
            })
            .map_err(|e| format!("Failed to spawn browser thread: {}", e))?;

        Ok(BrowserHandle { tx: cmd_tx, connected })
    }

    async fn run_cdp_loop(
        chrome: String,
        base_url: String,
        mut cmd_rx: mpsc::Receiver<BrowserCommand>,
        connected: Arc<RwLock<bool>>,
    ) {
        // 1. Launch headless chromium
        let cdp_port = match launch_chromium(&chrome).await {
            Ok(p) => p,
            Err(e) => { crate::log_fail!("Failed to launch chromium: {}", e); return; }
        };

        let cdp_url = format!("http://127.0.0.1:{}", cdp_port);
        crate::log_ok!("Chromium launched, CDP port: {}", cdp_port);

        // 2. Get WebSocket URL from HTTP endpoint
        let ws_url = match get_ws_url(&cdp_url).await {
            Ok(u) => u,
            Err(e) => { crate::log_fail!("Failed to get CDP WS URL: {}", e); return; }
        };

        // 3. Connect WebSocket
        let (ws_stream, _response) = match connect_async(&ws_url).await {
            Ok(s) => s,
            Err(e) => { crate::log_fail!("CDP WS connect failed: {}", e); return; }
        };

        let (mut write, mut read) = ws_stream.split();
        *connected.write().await = true;
        crate::log_ok!("Debug browser connected via CDP");

        // Enable Page domain
        let _ = send_cdp(&mut write, 1, "Page.enable", serde_json::Value::Null).await;

        let mut cmd_id: i32 = 100;

        loop {
            tokio::select! {
                cmd = cmd_rx.recv() => {
                    let Some(cmd) = cmd else { break; };
                    match cmd {
                        BrowserCommand::Navigate { url, resp } => {
                            cmd_id += 1;
                            let target = if url.starts_with("http") { url.clone() } else { format!("{}{}", base_url, url) };
                            match send_and_recv(&mut write, &mut read, cmd_id, "Page.navigate",
                                serde_json::json!({ "url": target })).await
                            {
                                Ok(_) => {
                                    tokio::time::sleep(Duration::from_millis(500)).await;
                                    cmd_id += 1;
                                    let title = send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate",
                                        serde_json::json!({"expression":"document.title","returnByValue":true})).await
                                        .ok()
                                        .and_then(|v| v.pointer("/result/result/value").and_then(|v| v.as_str()).map(String::from))
                                        .unwrap_or_default();
                                    let _ = resp.send(Ok(NavigateResponse { url: target, title }));
                                }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::Screenshot { selector, full_page, resp } => {
                            cmd_id += 1;
                            let mut params = serde_json::json!({"format": "png"});
                            if full_page { params["captureBeyondViewport"] = serde_json::json!(true); }
                            if let Some(sel) = &selector {
                                cmd_id += 1;
                                let js = format!(
                                    "(()=>{{var el=document.querySelector({:?});if(!el)return null;var r=el.getBoundingClientRect();return{{x:r.x,y:r.y,w:r.width,h:r.height,dpr:window.devicePixelRatio||1}}}})()", sel);
                                if let Ok(rv) = send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate",
                                    serde_json::json!({"expression":js,"returnByValue":true})).await
                                {
                                    if let Some(rect) = rv.pointer("/result/result/value") {
                                        if !rect.is_null() {
                                            let dpr = rect["dpr"].as_f64().unwrap_or(1.0);
                                            params["clip"] = serde_json::json!({
                                                "x": rect["x"], "y": rect["y"],
                                                "width": rect["w"], "height": rect["h"],
                                                "scale": dpr,
                                            });
                                        }
                                    }
                                }
                                cmd_id += 1;
                            }
                            match send_and_recv(&mut write, &mut read, cmd_id, "Page.captureScreenshot", params).await {
                                Ok(v) => {
                                    let data = v.pointer("/data").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    let _ = resp.send(Ok(ScreenshotResponse { data, mime_type: "image/png".into(), width: DEFAULT_VIEWPORT_W, height: DEFAULT_VIEWPORT_H }));
                                }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::Click { selector, resp } => {
                            cmd_id += 1;
                            let js = format!("(()=>{{var el=document.querySelector({:?});if(!el)return'error:not found';el.click();return 'ok'}})()", selector);
                            match send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate",
                                serde_json::json!({"expression":js,"returnByValue":true})).await
                            {
                                Ok(v) => {
                                    let s = v.pointer("/result/result/value").and_then(|v| v.as_str()).unwrap_or("");
                                    if s == "ok" { let _ = resp.send(Ok(())); }
                                    else { let _ = resp.send(Err(s.to_string())); }
                                }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::TypeText { selector, text, clear_first, submit, resp } => {
                            cmd_id += 1;
                            let js = format!(
                                "(()=>{{var el=document.querySelector({:?});if(!el)return'error:not found';if({})el.value='';var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(el,{:?});el.dispatchEvent(new Event('input',{{bubbles:true}}));el.dispatchEvent(new Event('change',{{bubbles:true}}));if({})el.form?.submit();return 'ok'}})()",
                                selector, clear_first, text, submit
                            );
                            match send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate",
                                serde_json::json!({"expression":js,"returnByValue":true})).await
                            {
                                Ok(v) => {
                                    let s = v.pointer("/result/result/value").and_then(|v| v.as_str()).unwrap_or("");
                                    if s == "ok" { let _ = resp.send(Ok(())); }
                                    else { let _ = resp.send(Err(s.to_string())); }
                                }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::Evaluate { expression, await_promise, resp } => {
                            cmd_id += 1;
                            let expr = if await_promise { format!("(async()=>{{return await({})}})()", expression) } else { expression.clone() };
                            let mut params = serde_json::json!({"expression": expr, "returnByValue": true});
                            if await_promise { params["awaitPromise"] = serde_json::json!(true); }
                            match send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate", params).await {
                                Ok(v) => { let _ = resp.send(parse_eval_result(&v)); }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::DomQuery { selector, attribute, resp } => {
                            cmd_id += 1;
                            let js = if let Some(attr) = attribute {
                                format!("(()=>{{var el=document.querySelector({:?});return el?.getAttribute({:?})??null}})()", selector, attr)
                            } else {
                                format!(
                                    "(()=>{{var els=document.querySelectorAll({:?});if(!els.length)return JSON.stringify({{error:'not found',count:0}});var el=els[0],r=el.getBoundingClientRect();return JSON.stringify({{tag:el.tagName.toLowerCase(),text:el.textContent?.trim()?.substring(0,2000)??null,html:el.outerHTML.substring(0,5000),attrs:Array.from(el.attributes).reduce((a,x)=>(a[x.name]=x.value,a),{{}}),visible:r.width>0&&r.height>0,count:els.length}})}})()",
                                    selector
                                )
                            };
                            match send_and_recv(&mut write, &mut read, cmd_id, "Runtime.evaluate",
                                serde_json::json!({"expression":js,"returnByValue":true})).await
                            {
                                Ok(v) => {
                                    let raw = v.pointer("/result/result/value").and_then(|v| v.as_str()).unwrap_or("{}");
                                    match serde_json::from_str::<DomNodeResponse>(raw) {
                                        Ok(dr) => { let _ = resp.send(Ok(dr)); }
                                        Err(_) => { let _ = resp.send(Err(format!("DOM parse: {}", raw))); }
                                    }
                                }
                                Err(e) => { let _ = resp.send(Err(e)); }
                            }
                        }
                        BrowserCommand::Shutdown => {
                            let _ = send_cdp(&mut write, 99999, "Browser.close", serde_json::Value::Null).await;
                            break;
                        }
                    }
                }
                msg = read.next() => {
                    if let Some(Ok(Message::Ping(_))) = msg {
                        let _ = write.send(Message::Pong(Vec::new())).await;
                    }
                }
            }
        }
        *connected.write().await = false;
        crate::log_info!("Debug browser disconnected");
    }

    /// Launch headless chromium, return its CDP port number.
    async fn launch_chromium(chrome_path: &str) -> Result<u16, String> {
        let mut child = std::process::Command::new(chrome_path)
            .args([
                "--headless=new", "--no-sandbox", "--disable-gpu",
                "--remote-debugging-port=0", "--disable-dev-shm-usage",
                "--window-size=1280,720", "--disable-extensions", "--no-first-run",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn {}: {}", chrome_path, e))?;

        let mut stderr = child.stderr.take().ok_or("No stderr")?;
        let path = chrome_path.to_string();

        let port: Option<u16> = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
            while std::time::Instant::now() < deadline {
                match std::io::Read::read(&mut stderr, &mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]);
                        for line in text.lines() {
                            if line.contains("DevTools listening on") {
                                if let Some(ps) = line.rsplit(':').next() {
                                    if let Some(pp) = ps.split('/').next() {
                                        if let Ok(p) = pp.parse::<u16>() { return Some(p); }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            None
        }).await.map_err(|e| e.to_string())?;

        let p = port.ok_or_else(|| "Chromium did not report DevTools port".to_string())?;
        std::mem::forget(child);
        Ok(p)
    }

    /// Get the WebSocket URL from CDP's HTTP endpoint.
    async fn get_ws_url(cdp_http: &str) -> Result<String, String> {
        let body = reqwest::get(format!("{}/json/version", cdp_http))
            .await.map_err(|e| e.to_string())?
            .text().await.map_err(|e| e.to_string())?;
        let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;
        v.get("webSocketDebuggerUrl")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| "No webSocketDebuggerUrl in CDP response".into())
    }

    async fn send_cdp<W>(write: &mut W, id: i32, method: &str, params: serde_json::Value) -> Result<(), String>
    where
        W: futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
    {
        let cmd = serde_json::json!({ "id": id, "method": method, "params": params });
        write.send(Message::Text(cmd.to_string())).await.map_err(|e| e.to_string())
    }

    async fn send_and_recv<W, R>(
        write: &mut W, read: &mut R,
        id: i32, method: &str, params: serde_json::Value,
    ) -> Result<serde_json::Value, String>
    where
        W: futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
        R: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        send_cdp(write, id, method, params).await?;
        recv_response(read, id).await
    }
    async fn recv_response<R>(
        read: &mut R, expected_id: i32,
    ) -> Result<serde_json::Value, String>
    where
        R: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin,
    {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(OP_TIMEOUT_SECS);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(100), read.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&text) {
                        if msg.get("id").and_then(|i| i.as_i64()) == Some(expected_id as i64) {
                            if msg.get("error").is_some() {
                                return Err(msg["error"]["message"].as_str().unwrap_or("CDP error").to_string());
                            }
                            return Ok(msg);
                        }
                    }
                }
                Ok(None) | Ok(Some(Err(_))) => return Err("WebSocket closed".into()),
                Err(_) => continue,
                Ok(Some(Ok(_))) => continue,
            }
        }
        Err("CDP response timeout".into())
    }

    fn parse_eval_result(v: &serde_json::Value) -> Result<EvaluateResponse, String> {
        let rv = v.pointer("/result/result").ok_or("No result")?;
        let type_name = match rv.get("type").and_then(|t| t.as_str()) {
            Some("undefined" | "null") => "null",
            Some("boolean") => "boolean",
            Some("number") => "number",
            Some("string") => "string",
            Some("object" | "function") => "object",
            _ => "unknown",
        };
        Ok(EvaluateResponse {
            result: rv.get("value").cloned().unwrap_or(serde_json::Value::Null),
            r#type: type_name.into(),
        })
    }
}

// ── Fallback when debug-browser feature is disabled ───────────────────────

#[cfg(not(feature = "debug-browser"))]
mod engine {
    use super::*;
    pub(super) fn spawn_browser(
        _base_url: String, _initial_url: Option<String>,
        _console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    ) -> Result<BrowserHandle, String> {
        Err("debug-browser feature not enabled".into())
    }
}

// ── DebugState ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct DebugState {
    config: Config,
    dev_port: u16,
    debug_port: u16,
    start_time: Instant,
    base_url: String,
    console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    browser: Option<Arc<BrowserHandle>>,
    browser_engine: String,
}

impl DebugState {
    fn new(config: Config, dev_port: u16, debug_port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", dev_port),
            config, dev_port, debug_port,
            start_time: Instant::now(),
            console_log: Arc::new(RwLock::new(Vec::new())),
            browser: None,
            browser_engine: "none".into(),
        }
    }
    fn uptime_secs(&self) -> u64 { self.start_time.elapsed().as_secs() }
}

// ── Server startup ───────────────────────────────────────────────────────

pub async fn start_debug_server(
    config: &Config, dev_port: u16, debug_port: u16,
) -> crate::Result<()> {
    let base_url = format!("http://localhost:{}", dev_port);
    let console_log = Arc::new(RwLock::new(Vec::new()));

    let browser = engine::spawn_browser(base_url.clone(), None, console_log.clone())
        .ok()
        .map(Arc::new);

    let browser_engine = if browser.is_some() {
        #[cfg(feature = "debug-browser")] { "cdp+chromium" }
        #[cfg(not(feature = "debug-browser"))] { "none" }
    } else {
        "none"
    }.to_string();

    let state = DebugState {
        config: config.clone(), dev_port, debug_port,
        base_url, console_log, browser, browser_engine,
        start_time: Instant::now(),
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], debug_port));

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler))
        .route("/navigate", post(navigate_handler))
        .route("/screenshot", post(screenshot_handler))
        .route("/click", post(click_handler))
        .route("/type", post(type_handler))
        .route("/evaluate", post(evaluate_handler))
        .route("/console", get(console_handler))
        .route("/dom", get(dom_query_handler))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    crate::log_ok!("Debug API listening on http://localhost:{}", debug_port);
    crate::log_info!("Endpoints: /health /info /navigate /screenshot /click /type /evaluate /console /dom");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// ── HTTP handlers ─────────────────────────────────────────────────────────

async fn health_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(HealthResponse {
        status: "ok".into(), version: crate::VERSION.into(),
        api_version: DEBUG_API_VERSION.into(), uptime_secs: state.uptime_secs(),
    }))
}

async fn info_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let bc = state.browser.as_ref().map_or(false, |b| futures::executor::block_on(b.is_connected()));
    ResponseJson(ApiResponse::ok(InfoResponse {
        version: crate::VERSION.into(), api_version: DEBUG_API_VERSION.into(),
        dev_port: state.dev_port, debug_port: state.debug_port,
        dist_dir: state.config.build.output_dir.display().to_string(),
        package_name: state.config.package.name.clone(),
        pid: std::process::id(),
        started_at_iso: chrono::Utc::now().to_rfc3339(),
        uptime_secs: state.uptime_secs(),
        browser_connected: bc, browser_engine: state.browser_engine.clone(),
    }))
}

async fn navigate_handler(State(state): State<DebugState>, Json(req): Json<NavigateRequest>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<NavigateResponse>() };
    let target = if req.url.starts_with("http") { req.url } else { format!("{}{}", state.base_url, req.url) };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Navigate { url: target, resp: tx }).await.is_err() { return chan_closed::<NavigateResponse>(); }
    await_op(rx).await
}

async fn screenshot_handler(State(state): State<DebugState>, Json(params): Json<ScreenshotParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<ScreenshotResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Screenshot { selector: params.selector, full_page: params.full_page.unwrap_or(false), resp: tx }).await.is_err() { return chan_closed::<ScreenshotResponse>(); }
    await_op(rx).await
}

async fn click_handler(State(state): State<DebugState>, Json(req): Json<ClickRequest>) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Click { selector: req.selector, resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn type_handler(State(state): State<DebugState>, Json(req): Json<TypeRequest>) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::TypeText { selector: req.selector, text: req.text, clear_first: req.clear_first.unwrap_or(true), submit: req.submit.unwrap_or(false), resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn evaluate_handler(State(state): State<DebugState>, Json(req): Json<EvaluateRequest>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<EvaluateResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Evaluate { expression: req.expression, await_promise: req.await_promise.unwrap_or(false), resp: tx }).await.is_err() { return chan_closed::<EvaluateResponse>(); }
    await_op(rx).await
}

async fn console_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(ConsoleResponse { entries: state.console_log.read().await.clone() }))
}

async fn dom_query_handler(State(state): State<DebugState>, Query(params): Query<DomQueryParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<DomNodeResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::DomQuery { selector: params.selector, attribute: params.attribute, resp: tx }).await.is_err() { return chan_closed::<DomNodeResponse>(); }
    await_op(rx).await
}

// ── Helpers ───────────────────────────────────────────────────────────────

fn svc_unavailable<T: Serialize>() -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    (StatusCode::SERVICE_UNAVAILABLE, ResponseJson(ApiResponse::<T>::err("No browser available")))
}
fn chan_closed<T: Serialize>() -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    (StatusCode::SERVICE_UNAVAILABLE, ResponseJson(ApiResponse::<T>::err("Browser channel closed")))
}

async fn await_op<T: Serialize>(rx: oneshot::Receiver<Result<T, String>>) -> (StatusCode, ResponseJson<ApiResponse<T>>) {
    match tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await {
        Ok(Ok(Ok(d))) => (StatusCode::OK, ResponseJson(ApiResponse::ok(d))),
        Ok(Ok(Err(e))) => (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::err(e))),
        Ok(Err(_)) => chan_closed::<T>(),
        Err(_) => (StatusCode::GATEWAY_TIMEOUT, ResponseJson(ApiResponse::err("Operation timed out"))),
    }
}
