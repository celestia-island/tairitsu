use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    Router,
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc, oneshot};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;

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
#[allow(dead_code)]
struct ScreenshotParams {
    selector: Option<String>,
    full_page: Option<bool>,
    format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenshotResponse {
    data: String,
    mime_type: String,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ClickRequest {
    selector: String,
    button: Option<String>,
    modifiers: Option<Vec<String>>,
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
    modifiers: Option<Vec<String>>,
    count: Option<u32>,
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

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
struct NetworkQueryParams {
    limit: Option<usize>,
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
        computed: Option<Vec<String>>,
        resp: oneshot::Sender<Result<DomNodeResponse, String>>,
    },
    IsReady {
        resp: oneshot::Sender<Result<ReadyResponse, String>>,
    },
    Press {
        key: String,
        modifiers: Vec<String>,
        count: u32,
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
    #[allow(dead_code)]
    Shutdown,
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

// ── Chromium-based Browser Engine (CDP) ─────────────────────────────────────

#[cfg(feature = "debug-browser")]
mod engine {
    use super::*;
    use base64::Engine;
    use chromiumoxide::browser::{Browser, BrowserConfig};
    use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
    use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
    use chromiumoxide::page::ScreenshotParams;
    use futures::StreamExt;

    pub(super) async fn spawn_browser(
        base_url: String,
        _initial_url: Option<String>,
        _console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    ) -> Result<BrowserHandle, String> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<BrowserCommand>(64);
        let connected = Arc::new(RwLock::new(false));
        let conn = connected.clone();

        let config = resolve_browser_config().await?;

        let _ = std::fs::remove_file("/tmp/chromiumoxide-runner/SingletonLock");

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| format!("Failed to launch Chrome: {e}"))?;

        let _handler_guard = tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if event.is_err() {
                    break;
                }
            }
        });

        let page = browser
            .new_page(&base_url)
            .await
            .map_err(|e| format!("Failed to create page: {e}"))?;

        *connected.write().await = true;
        crate::log_ok!("Debug browser connected (chromium CDP)");

        let page = Arc::new(page);
        let browser = Arc::new(browser);

        tokio::spawn({
            let browser = browser.clone();
            async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    let p = page.clone();
                    tokio::spawn(async move {
                        dispatch_command(&p, cmd).await;
                    });
                }
                *conn.write().await = false;
                drop(browser);
            }
        });

        Ok(BrowserHandle {
            tx: cmd_tx,
            connected,
        })
    }

    async fn resolve_browser_config() -> Result<BrowserConfig, String> {
        let mut builder = BrowserConfig::builder()
            .window_size(DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H)
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu");

        if let Ok(exe) = std::env::var("CHROME_PATH")
            && !exe.is_empty()
        {
            crate::log_info!("[debug-browser] Using CHROME_PATH={}", exe);
            builder = builder.chrome_executable(exe);
            return builder
                .build()
                .map_err(|e| format!("Bad browser config: {e}"));
        }

        if let Ok(exe) = which_chromium() {
            crate::log_info!("[debug-browser] Found browser: {}", exe);
            builder = builder.chrome_executable(exe);
            return builder
                .build()
                .map_err(|e| format!("Bad browser config: {e}"));
        }

        crate::log_info!("[debug-browser] No browser found, auto-downloading Chromium...");
        let fetcher = chromiumoxide::fetcher::BrowserFetcher::new(
            chromiumoxide::fetcher::BrowserFetcherOptions::builder()
                .build()
                .map_err(|e| format!("Fetcher config: {e}"))?,
        );
        let info = fetcher
            .fetch()
            .await
            .map_err(|e| format!("Fetcher download: {e}"))?;
        crate::log_ok!(
            "[debug-browser] Chromium downloaded: {}",
            info.executable_path.display()
        );
        builder = builder.chrome_executable(&info.executable_path);
        builder
            .build()
            .map_err(|e| format!("Bad browser config: {e}"))
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
            if let Ok(output) = std::process::Command::new("which").arg(name).output()
                && output.status.success()
            {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }
        Err(())
    }

    async fn dispatch_command(page: &chromiumoxide::Page, cmd: BrowserCommand) {
        match cmd {
            BrowserCommand::Navigate {
                url,
                wait_for,
                resp,
            } => {
                let r = cmd_navigate(page, &url, wait_for.as_deref()).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Screenshot {
                selector,
                full_page,
                resp,
            } => {
                let r = cmd_screenshot(page, selector.as_deref(), full_page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Click { selector, resp } => {
                let r = cmd_click(page, &selector).await;
                let _ = resp.send(r);
            }
            BrowserCommand::TypeText {
                selector,
                text,
                clear_first,
                submit,
                resp,
            } => {
                let r = cmd_type(page, &selector, &text, clear_first, submit).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Evaluate {
                expression,
                await_promise,
                resp,
            } => {
                let r = cmd_evaluate(page, &expression, await_promise).await;
                let _ = resp.send(r);
            }
            BrowserCommand::DomQuery {
                selector,
                attribute,
                computed,
                resp,
            } => {
                let r =
                    cmd_dom_query(page, &selector, attribute.as_deref(), computed.as_deref()).await;
                let _ = resp.send(r);
            }
            BrowserCommand::IsReady { resp } => {
                let r = cmd_is_ready(page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Press { key, resp, .. } => {
                let r = cmd_press(page, &key).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Scroll {
                selector,
                x,
                y,
                resp,
            } => {
                let r = cmd_scroll(page, selector.as_deref(), x, y).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Resize {
                width,
                height,
                resp,
            } => {
                let r = cmd_resize(page, width, height).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Viewport { resp } => {
                let r = cmd_viewport(page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::A11y {
                selector,
                depth,
                resp,
            } => {
                let r = cmd_a11y(page, selector.as_deref(), depth).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Network { resp } => {
                let r = cmd_network(page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Performance { resp } => {
                let r = cmd_performance(page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Drag {
                from_selector,
                to_selector,
                steps,
                resp,
            } => {
                let r = cmd_drag(page, &from_selector, &to_selector, steps).await;
                let _ = resp.send(r);
            }
            BrowserCommand::WebSocket { resp } => {
                let r = cmd_websocket(page).await;
                let _ = resp.send(r);
            }
            BrowserCommand::Shutdown => {}
        }
    }

    async fn cmd_navigate(
        page: &chromiumoxide::Page,
        url: &str,
        wait_for: Option<&str>,
    ) -> Result<NavigateResponse, String> {
        page.goto(url).await.map_err(|e| format!("navigate: {e}"))?;
        if matches!(wait_for, Some("hydration") | Some("ready")) {
            tokio::time::sleep(Duration::from_secs(3)).await;
        } else if matches!(wait_for, Some("load")) {
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        Ok(NavigateResponse {
            url: url.to_string(),
            title,
        })
    }

    async fn cmd_screenshot(
        page: &chromiumoxide::Page,
        selector: Option<&str>,
        full_page: bool,
    ) -> Result<ScreenshotResponse, String> {
        if let Some(sel) = selector {
            let element = page
                .find_element(sel)
                .await
                .map_err(|e| format!("element not found: {e}"))?;
            let png = element
                .screenshot(CaptureScreenshotFormat::Png)
                .await
                .map_err(|e| format!("screenshot element: {e}"))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
            return Ok(ScreenshotResponse {
                data: b64,
                mime_type: "image/png".into(),
                width: 0,
                height: 0,
            });
        }
        let mut params = ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .omit_background(false);
        if full_page {
            params = params.full_page(true);
        }
        let png = page
            .screenshot(params.build())
            .await
            .map_err(|e| format!("screenshot: {e}"))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        Ok(ScreenshotResponse {
            data: b64,
            mime_type: "image/png".into(),
            width: DEFAULT_VIEWPORT_W,
            height: DEFAULT_VIEWPORT_H,
        })
    }

    async fn cmd_click(page: &chromiumoxide::Page, selector: &str) -> Result<(), String> {
        page.find_element(selector)
            .await
            .map_err(|e| format!("click: element not found: {e}"))?
            .click()
            .await
            .map_err(|e| format!("click: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_type(
        page: &chromiumoxide::Page,
        selector: &str,
        text: &str,
        clear_first: bool,
        _submit: bool,
    ) -> Result<(), String> {
        let el = page
            .find_element(selector)
            .await
            .map_err(|e| format!("type: element not found: {e}"))?;
        el.click().await.map_err(|e| format!("type click: {e}"))?;
        if clear_first {
            let js = format!(
                r#"(() => {{ const el = document.querySelector({selector:?}); if (el) {{ el.value = ''; el.dispatchEvent(new Event('input', {{bubbles: true}})); }} }})()"#,
            );
            page.evaluate(js)
                .await
                .map_err(|e| format!("type clear: {e}"))?;
        }
        el.type_str(text).await.map_err(|e| format!("type: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_evaluate(
        page: &chromiumoxide::Page,
        expression: &str,
        _await_promise: bool,
    ) -> Result<EvaluateResponse, String> {
        let result = page
            .evaluate(expression.to_string())
            .await
            .map_err(|e| format!("evaluate: {e}"))?;

        let val: serde_json::Value = result.into_value().unwrap_or(serde_json::Value::Null);

        let type_name = match &val {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) | serde_json::Value::Object(_) => "object",
        };
        Ok(EvaluateResponse {
            result: val,
            r#type: type_name.into(),
        })
    }

    async fn cmd_dom_query(
        page: &chromiumoxide::Page,
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
            let val = page
                .evaluate(js)
                .await
                .map_err(|e| format!("dom query: {e}"))?;
            let r: Option<String> = val.into_value().ok();
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
            });
        }
        let js = format!(
            r#"(() => {{ const els = document.querySelectorAll({sel:?}); if (!els.length) throw 'not found'; const el = els[0]; const r = el.getBoundingClientRect(); return JSON.stringify({{ tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 2000), html: el.outerHTML.substring(0, 5000), attrs: Object.fromEntries(Array.from(el.attributes).map(a => [a.name, a.value])), visible: r.width > 0 && r.height > 0, count: els.length, rect: {{ x: r.x, y: r.y, width: r.width, height: r.height }} }}); }})()"#,
            sel = selector,
        );
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("dom query: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("dom query parse: {e}"))?;
        serde_json::from_str::<DomNodeResponse>(&json_str)
            .map_err(|e| format!("dom query deserialize: {e}"))
    }

    async fn cmd_is_ready(page: &chromiumoxide::Page) -> Result<ReadyResponse, String> {
        let js = r#"(() => { const w = !!globalThis.__wasmExports; const h = document.documentElement.dataset.tairitsuReady === 'hydrated'; return JSON.stringify({ ready: w && h, wasm_loaded: w, hydrated: h, url: location.href }); })()"#;
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("is_ready: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("is_ready parse: {e}"))?;
        serde_json::from_str::<ReadyResponse>(&json_str)
            .map_err(|e| format!("is_ready deserialize: {e}"))
    }

    async fn cmd_press(page: &chromiumoxide::Page, key: &str) -> Result<(), String> {
        let js = format!(
            r#"(() => {{ document.dispatchEvent(new KeyboardEvent('keydown', {{key: {key:?}, code: {key:?}, bubbles: true}})); document.dispatchEvent(new KeyboardEvent('keyup', {{key: {key:?}, code: {key:?}, bubbles: true}})); }})()"#,
        );
        page.evaluate(js).await.map_err(|e| format!("press: {e}"))?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    async fn cmd_scroll(
        page: &chromiumoxide::Page,
        selector: Option<&str>,
        x: f64,
        y: f64,
    ) -> Result<(), String> {
        let js = if let Some(sel) = selector {
            format!(
                r#"(() => {{ const el = document.querySelector({sel:?}); if (el) el.scrollBy({x}, {y}); }})()"#,
            )
        } else {
            format!(r#"window.scrollBy({x}, {y})"#)
        };
        page.evaluate(js)
            .await
            .map_err(|e| format!("scroll: {e}"))?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn cmd_resize(page: &chromiumoxide::Page, width: u32, height: u32) -> Result<(), String> {
        let params = SetDeviceMetricsOverrideParams::builder()
            .width(width as i64)
            .height(height as i64)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
            .map_err(|e| format!("resize build: {e}"))?;
        page.execute(params)
            .await
            .map_err(|e| format!("resize: {e}"))?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    async fn cmd_viewport(page: &chromiumoxide::Page) -> Result<ViewportResponse, String> {
        let js = r#"(() => { const dpr = window.devicePixelRatio || 1; return JSON.stringify({ width: window.innerWidth, height: window.innerHeight, device_pixel_ratio: dpr }); })()"#;
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("viewport: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("viewport parse: {e}"))?;
        serde_json::from_str::<ViewportResponse>(&json_str)
            .map_err(|e| format!("viewport deserialize: {e}"))
    }

    async fn cmd_a11y(
        page: &chromiumoxide::Page,
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
"#.replace("SEL_JS", &sel_js)
           .replace("DEPTH", &depth.to_string());

        let val = page
            .evaluate(js_body)
            .await
            .map_err(|e| format!("a11y: {e}"))?;
        let json_str: String = val.into_value().map_err(|e| format!("a11y parse: {e}"))?;
        serde_json::from_str::<Vec<A11yNode>>(&json_str)
            .map_err(|e| format!("a11y deserialize: {e}"))
    }

    async fn cmd_network(page: &chromiumoxide::Page) -> Result<NetworkResponse, String> {
        let js = r#"(() => { var entries = performance.getEntriesByType('resource').slice(0, 100).map(function(e) { return { name: e.name, type: e.initiatorType || 'unknown', duration: Math.round(e.duration * 100) / 100, size: e.transferSize || 0, url: e.name }; }); return JSON.stringify({ resources: entries }); })()"#;
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("network: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("network parse: {e}"))?;
        serde_json::from_str::<NetworkResponse>(&json_str)
            .map_err(|e| format!("network deserialize: {e}"))
    }

    async fn cmd_performance(page: &chromiumoxide::Page) -> Result<PerformanceMetrics, String> {
        let js = r#"(() => { var nav = performance.getEntriesByType('navigation')[0] || {}; var fcp = null; try { fcp = performance.getEntriesByName('first-contentful-paint')[0].startTime || null; } catch(e) {} var dn = document.querySelectorAll('*').length; var heap = null; try { heap = Math.round((performance.memory ? performance.memory.usedJSHeapSize : 0) / 1048576 * 100) / 100; } catch(e) {} return JSON.stringify({ dom_content_loaded_ms: Math.round((nav.domContentLoadedEventEnd - nav.startTime) * 100) / 100 || null, dom_complete_ms: Math.round((nav.domComplete - nav.startTime) * 100) / 100 || null, load_event_ms: Math.round((nav.loadEventEnd - nav.startTime) * 100) / 100 || null, fcp_ms: fcp ? Math.round(fcp * 100) / 100 : null, lcp_ms: null, cls: null, dom_nodes: dn, js_heap_used_mb: heap, wasm_loaded: !!globalThis.__wasmExports, hydrated: document.documentElement.dataset.tairitsuReady === 'hydrated', timestamp: new Date().toISOString() }); })()"#;
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("performance: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("performance parse: {e}"))?;
        serde_json::from_str::<PerformanceMetrics>(&json_str)
            .map_err(|e| format!("performance deserialize: {e}"))
    }

    async fn cmd_drag(
        page: &chromiumoxide::Page,
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
        page.evaluate(js).await.map_err(|e| format!("drag: {e}"))?;
        tokio::time::sleep(Duration::from_millis(200)).await;
        Ok(())
    }

    async fn cmd_websocket(page: &chromiumoxide::Page) -> Result<WebSocketInfo, String> {
        let js = r#"(() => { var c = 0; var conns = []; var t = window._wsTracker || []; t.forEach(function(ws) { c++; conns.push({ url: ws.url || 'unknown', state: ws.readyState === 0 ? 'connecting' : ws.readyState === 1 ? 'open' : ws.readyState === 2 ? 'closing' : 'closed', created_at_ms: null }); }); return JSON.stringify({ active_count: c, connections: conns }); })()"#;
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("websocket: {e}"))?;
        let json_str: String = val
            .into_value()
            .map_err(|e| format!("websocket parse: {e}"))?;
        serde_json::from_str::<WebSocketInfo>(&json_str)
            .map_err(|e| format!("websocket deserialize: {e}"))
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
    errors: Arc<RwLock<Vec<ErrorEntry>>>,
    rejections: Arc<RwLock<Vec<ErrorEntry>>>,
    browser: Option<Arc<BrowserHandle>>,
    browser_engine: String,
}

impl DebugState {
    #[allow(dead_code)]
    fn new(config: Config, dev_port: u16, debug_port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", dev_port),
            config,
            dev_port,
            debug_port,
            start_time: Instant::now(),
            console_log: Arc::new(RwLock::new(Vec::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
            rejections: Arc::new(RwLock::new(Vec::new())),
            browser: None,
            browser_engine: "none".into(),
        }
    }
    fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

// ── Server startup ───────────────────────────────────────────────────────

pub async fn start_debug_server(
    config: &Config,
    dev_port: u16,
    debug_port: u16,
) -> crate::Result<()> {
    let base_url = format!("http://localhost:{}", dev_port);
    let console_log = Arc::new(RwLock::new(Vec::new()));

    #[cfg(feature = "debug-browser")]
    let (browser, browser_engine) = {
        crate::log_info!("Debug browser engine: chromium (headless CDP)");
        match tokio::time::timeout(
            Duration::from_secs(30),
            engine::spawn_browser(base_url.clone(), None, console_log.clone()),
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
        config: config.clone(),
        dev_port,
        debug_port,
        base_url,
        console_log,
        errors: Arc::new(RwLock::new(Vec::new())),
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
        dist_dir: state.config.build.output_dir.display().to_string(),
        package_name: state.config.package.name.clone(),
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
            modifiers: req.modifiers.unwrap_or_default(),
            count: req.count.unwrap_or(1),
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
            if let Some(ref src) = params.source
                && e.source.as_deref() != Some(src.as_str())
            {
                return false;
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
            if let Some(colon2) = before_col.rfind(':')
                && let Ok(line) = before_col[colon2 + 1..].parse::<u32>()
            {
                return (before_col[..colon2].to_string(), Some(line), Some(col));
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
