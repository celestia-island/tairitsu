use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tower_http::compression::CompressionLayer;

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
    fn ok(data: T) -> Self { Self { ok: true, data: Some(data), error: None } }
    fn err(msg: impl Into<String>) -> Self { Self { ok: false, data: None, error: Some(msg.into()) } }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HealthResponse { status: String, version: String, api_version: String, uptime_secs: u64 }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InfoResponse {
    version: String, api_version: String, dev_port: u16, debug_port: u16,
    dist_dir: String, package_name: String, pid: u32,
    started_at_iso: String, uptime_secs: u64,
    browser_connected: bool, browser_engine: String, viewport: [u32; 2],
}

#[derive(Debug, Clone, Deserialize)]
struct NavigateRequest { url: String, wait_for: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigateResponse { url: String, title: String }

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
struct ScreenshotParams { selector: Option<String>, full_page: Option<bool>, format: Option<String>, mode: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenshotResponse { data: String, mime_type: String, width: u32, height: u32, mode: String }

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
struct ClickRequest { selector: String, button: Option<String>, modifiers: Option<Vec<String>> }

#[derive(Debug, Clone, Deserialize)]
struct TypeRequest { selector: String, text: String, clear_first: Option<bool>, submit: Option<bool> }

#[derive(Debug, Clone, Deserialize)]
struct EvaluateRequest { expression: String, await_promise: Option<bool> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvaluateResponse { result: serde_json::Value, r#type: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsoleEntry { level: String, text: String, timestamp: String, source: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConsoleResponse { entries: Vec<ConsoleEntry> }

#[derive(Debug, Clone, Deserialize, Default)]
struct DomQueryParams { selector: String, attribute: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DomNodeResponse {
    tag: Option<String>, text: Option<String>, html: Option<String>,
    attributes: Option<serde_json::Map<String, serde_json::Value>>,
    visible: Option<bool>, count: usize,
    rect: Option<RectResponse>, computed: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RectResponse { x: f64, y: f64, width: f64, height: f64, children_visible: Option<usize>, overflowing: Option<bool> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReadyResponse { ready: bool, wasm_loaded: bool, hydrated: bool, url: String }

#[derive(Debug, Clone, Deserialize)]
struct ComputedStyleParams { selector: String, properties: Option<Vec<String>> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ComputedStyleResponse { selector: String, properties: serde_json::Map<String, serde_json::Value> }

#[derive(Debug, Clone, Deserialize)]
struct PressRequest { key: String, modifiers: Option<Vec<String>>, count: Option<u32> }

#[derive(Debug, Clone, Deserialize)]
struct ScrollRequest { selector: Option<String>, x: Option<f64>, y: Option<f64>, direction: Option<String>, amount: Option<f64> }

#[derive(Debug, Clone, Deserialize)]
struct ResizeRequest { width: Option<u32>, height: Option<u32>, preset: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewportResponse { width: u32, height: u32, device_pixel_ratio: f64 }

#[derive(Debug, Clone, Deserialize, Default)]
struct ConsoleQueryParams { level: Option<String>, source: Option<String>, limit: Option<usize> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEntry { message: String, stack: Option<String>, r#type: String, timestamp: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorsResponse { errors: Vec<ErrorEntry>, unhandled_rejections: Vec<ErrorEntry> }

#[derive(Debug, Clone, Deserialize)]
struct DragRequest { from_selector: String, to_selector: String, steps: Option<u32> }

#[derive(Debug, Clone, Deserialize, Default)]
struct A11yQueryParams { selector: Option<String>, depth: Option<u32> }

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
struct BatchRequest { operations: Vec<BatchOperation> }

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
enum BatchOperation {
    #[serde(rename = "navigate")]
    Navigate { url: String, wait_for: Option<String> },
    #[serde(rename = "screenshot")]
    Screenshot { selector: Option<String>, full_page: Option<bool>, mode: Option<String>, name: Option<String> },
    #[serde(rename = "click")]
    Click { selector: String },
    #[serde(rename = "evaluate")]
    Evaluate { expression: String },
    #[serde(rename = "wait")]
    Wait { ms: u64 },
    #[serde(rename = "scroll")]
    Scroll { selector: Option<String>, direction: Option<String>, amount: Option<f64> },
    #[serde(rename = "resize")]
    Resize { width: Option<u32>, height: Option<u32>, preset: Option<String> },
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
struct NetworkQueryParams { limit: Option<usize> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkResource { name: String, r#type: String, duration: f64, size: f64, url: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkResponse { resources: Vec<NetworkResource> }

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
struct WebSocketInfo { active_count: u32, connections: Vec<WebSocketConn> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebSocketConn { url: String, state: String, created_at_ms: Option<f64> }

#[derive(Debug, Clone, Deserialize)]
struct SourceMapRequest { stack: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceMapResponse { frames: Vec<StackFrame>, raw: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StackFrame { file: String, line: Option<u32>, col: Option<u32>, func: Option<String>, raw: String }

// ── Browser command channel ───────────────────────────────────────────────

enum BrowserCommand {
    Navigate { url: String, wait_for: Option<String>, resp: oneshot::Sender<Result<NavigateResponse, String>> },
    Screenshot { selector: Option<String>, full_page: bool, mode: String, resp: oneshot::Sender<Result<ScreenshotResponse, String>> },
    Click { selector: String, resp: oneshot::Sender<Result<(), String>> },
    TypeText { selector: String, text: String, clear_first: bool, submit: bool, resp: oneshot::Sender<Result<(), String>> },
    Evaluate { expression: String, await_promise: bool, resp: oneshot::Sender<Result<EvaluateResponse, String>> },
    DomQuery { selector: String, attribute: Option<String>, computed: Option<Vec<String>>, resp: oneshot::Sender<Result<DomNodeResponse, String>> },
    IsReady { resp: oneshot::Sender<Result<ReadyResponse, String>> },
    Press { key: String, modifiers: Vec<String>, count: u32, resp: oneshot::Sender<Result<(), String>> },
    Scroll { selector: Option<String>, x: f64, y: f64, resp: oneshot::Sender<Result<(), String>> },
    Resize { width: u32, height: u32, resp: oneshot::Sender<Result<(), String>> },
    Viewport { resp: oneshot::Sender<Result<ViewportResponse, String>> },
    Drag { from_selector: String, to_selector: String, steps: u32, resp: oneshot::Sender<Result<(), String>> },
    A11y { selector: Option<String>, depth: u32, resp: oneshot::Sender<Result<Vec<A11yNode>, String>> },
    Network { resp: oneshot::Sender<Result<NetworkResponse, String>> },
    Performance { resp: oneshot::Sender<Result<PerformanceMetrics, String>> },
    WebSocket { resp: oneshot::Sender<Result<WebSocketInfo, String>> },
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
    async fn is_connected(&self) -> bool { *self.connected.read().await }
}

// ── Wry-based Browser Engine ─────────────────────────────────────────────

#[cfg(feature = "debug-browser")]
mod engine {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc as StdArc, Mutex};

    type PendingCallback = Box<dyn Send + FnOnce(Result<String, String>)>;
    type PendingMap = StdArc<Mutex<HashMap<i32, PendingCallback>>>;

    pub(super) fn spawn_browser(
        base_url: String,
        _initial_url: Option<String>,
        _console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    ) -> Result<BrowserHandle, String> {
        let (cmd_tx, cmd_rx) = mpsc::channel::<BrowserCommand>(64);
        let connected = Arc::new(RwLock::new(false));
        let conn = connected.clone();

        crate::log_info!("Debug browser engine: wry (cross-platform WebView)");

        std::thread::Builder::new()
            .name("tairitsu-debug-wry".into())
            .spawn(move || {
                run_wry_engine(base_url, cmd_rx, conn);
            })
            .map_err(|e| format!("Failed to spawn browser thread: {}", e))?;

        Ok(BrowserHandle { tx: cmd_tx, connected })
    }

    fn run_wry_engine(
        base_url: String,
        mut cmd_rx: mpsc::Receiver<BrowserCommand>,
        connected: Arc<RwLock<bool>>,
    ) {
        use tao::event::{Event, WindowEvent};
        use tao::event_loop::{ControlFlow, EventLoopBuilder};
        use tao::platform::unix::EventLoopBuilderExtUnix;
        use tao::window::WindowBuilder;
        use wry::WebViewBuilder;

        crate::log_info!("[wry] Creating event loop...");
        let event_loop = EventLoopBuilder::<BrowserCommand>::with_user_event()
            .with_any_thread(true)
            .build();
        let proxy = event_loop.create_proxy();

        std::thread::spawn(move || {
            while let Some(cmd) = cmd_rx.blocking_recv() {
                if proxy.send_event(cmd).is_err() { break; }
            }
        });

        crate::log_info!("[wry] Creating offscreen window...");
        let window = match WindowBuilder::new()
            .with_visible(true)
            .with_inner_size(tao::dpi::LogicalSize::new(DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H))
            .with_title("Tairitsu Debug Browser")
            .build(&event_loop)
        {
            Ok(w) => w,
            Err(e) => { crate::log_fail!("[wry] Failed to create window: {}", e); return; }
        };
        crate::log_info!("[wry] Window created OK");

        let pending: PendingMap = StdArc::new(Mutex::new(HashMap::new()));
        let pending_ipc = pending.clone();

        crate::log_info!("[wry] Creating WebView with URL {}...", base_url);
        #[cfg(target_os = "linux")]
        {
            unsafe {
                std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
                std::env::set_var("WEBKIT_FORCE_SOFTWARE_RENDERER", "1");
            }
        }
        let webview = match WebViewBuilder::new()
            .with_url(&base_url)
            .with_ipc_handler(move |request| {
                let body = request.body();
                if let Ok(msg) = serde_json::from_str::<serde_json::Value>(body) {
                    let id = msg.get("id").and_then(|v| v.as_i64()).unwrap_or(-1) as i32;
                    let mut map = pending_ipc.lock().unwrap();
                    if let Some(cb) = map.remove(&id) {
                        if msg.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                            let data = msg.get("data").and_then(|v| v.as_str()).unwrap_or("").to_string();
                            cb(Ok(data));
                        } else {
                            let err = msg.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string();
                            cb(Err(err));
                        }
                    }
                }
            })
            .build(&window)
        {
            Ok(wv) => wv,
            Err(e) => { crate::log_fail!("[wry] Failed to create WebView: {}", e); return; }
        };
        crate::log_info!("[wry] WebView created OK");

        *connected.blocking_write() = true;
        crate::log_ok!("Debug browser connected via wry");

        let mut next_id: i32 = 1;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::UserEvent(cmd) => {
                    match cmd {
                        BrowserCommand::Navigate { url, wait_for, resp } => {
                            let target = if url.starts_with("http") { url.clone() } else { format!("{}{}", base_url, url) };
                            let js = format!("window.location.href={:?}", target);
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(500));
                            if matches!(wait_for.as_deref(), Some("hydration") | Some("ready")) {
                                for _ in 0..15 {
                                    std::thread::sleep(Duration::from_millis(300));
                                }
                            } else if matches!(wait_for.as_deref(), Some("load")) {
                                std::thread::sleep(Duration::from_millis(500));
                            }
                            let _ = resp.send(Ok(NavigateResponse { url: target, title: String::new() }));
                        }
                        BrowserCommand::Screenshot { selector, full_page, mode, resp } => {
                            if mode == "pixel" {
                                match capture_webkit_snapshot(&webview) {
                                    Ok((png_bytes, w, h)) => {
                                        let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                                        let _ = resp.send(Ok(ScreenshotResponse { data: b64, mime_type: "image/png".into(), width: w, height: h, mode: "pixel".into() }));
                                        return;
                                    }
                                    Err(e) => {
                                        crate::log_fail!("[screenshot] webkit snapshot failed: {}, falling back to canvas", e);
                                    }
                                }
                            }
                            let id = next_id; next_id += 1;
                            let eval_js = build_screenshot_eval_js(selector.as_deref(), full_page);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => { let _ = r.send(Ok(ScreenshotResponse { data, mime_type: "image/png".into(), width: DEFAULT_VIEWPORT_W, height: DEFAULT_VIEWPORT_H, mode: "canvas".into() })); }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let wrapper = format!(
                                r#"(()=>{{try{{var r=({eval_js});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:r}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
                            );
                            let _ = webview.evaluate_script(&wrapper);
                        }
                        BrowserCommand::Click { selector, resp } => {
                            let js = format!(r#"(()=>{{var el=document.querySelector({:?});if(!el)return;el.click()}})()"#, selector);
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(100));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::TypeText { selector, text, clear_first, submit, resp } => {
                            let js = format!(
                                r#"(()=>{{var el=document.querySelector({:?});if(!el)return;if({})el.value='';var s=Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype,'value').set;s.call(el,{:?});el.dispatchEvent(new Event('input',{{bubbles:true}}));el.dispatchEvent(new Event('change',{{bubbles:true}}));if({})el.form?.submit()}})()"#,
                                selector, clear_first, text, submit
                            );
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(100));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::Evaluate { expression, await_promise, resp } => {
                            let id = next_id; next_id += 1;
                            let e = expression.replace('\\', "\\\\").replace('`', "\\`");
                            let js = if await_promise {
                                format!(r#"(async()=>{{try{{var r=await({e});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:r==null?null:typeof r==='object'?JSON.stringify(r):String(r)}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#)
                            } else {
                                format!(r#"(()=>{{try{{var r=({e});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:r==null?null:typeof r==='object'?JSON.stringify(r):String(r)}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#)
                            };
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        let (result_val, type_name) = parse_eval_result(&data);
                                        let _ = r.send(Ok(EvaluateResponse { result: result_val, r#type: type_name.into() }));
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::DomQuery { selector, attribute, computed, resp } => {
                            let id = next_id; next_id += 1;
                            let computed_js = if let Some(ref props) = computed {
                                let arr = serde_json::to_string(props).unwrap_or_else(|_| "[]".into());
                                format!("var cprops={};var cs=getComputedStyle(el);var cdata={{}};cprops.forEach(function(p){{cdata[p]=cs.getPropertyValue(p)}});", arr)
                            } else {
                                String::new()
                            };
                            let js = if let Some(attr) = &attribute {
                                format!(
                                    r#"(()=>{{try{{var el=document.querySelector({:?});var r=el?.getAttribute({:?})??null;window.ipc.postMessage(JSON.stringify({{id:{},ok:true,data:JSON.stringify({{tag:el?.tagName?.toLowerCase(),text:r,count:el?1:0}})}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{},ok:false,error:e.message}}))}}}})()"#,
                                    selector, attr, id, id
                                )
                            } else {
                                format!(
                                    r#"(()=>{{try{{var els=document.querySelectorAll({:?});if(!els.length)throw'not found';var el=els[0],r=el.getBoundingClientRect();{computed_js}var d={{tag:el.tagName.toLowerCase(),text:el.textContent?.trim()?.substring(0,2000)??null,html:el.outerHTML.substring(0,5000),attrs:Array.from(el.attributes).reduce((a,x)=>(a[x.name]=x.value,a),{{}}),visible:r.width>0&&r.height>0,count:els.length,rect:{{x:r.x,y:r.y,width:r.width,height:r.height}}}};window.ipc.postMessage(JSON.stringify({{id:{},ok:true,data:JSON.stringify(d)}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{},ok:false,error:String(e)}}))}}}})()"#,
                                    selector, id, id
                                )
                            };
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<DomNodeResponse>(&data) {
                                            Ok(dr) => { let _ = r.send(Ok(dr)); }
                                            Err(_) => { let _ = r.send(Err(format!("DOM parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::IsReady { resp } => {
                            let id = next_id; next_id += 1;
                            let js = format!(r#"(()=>{{try{{var w=!!globalThis.__wasmExports;var h=document.documentElement.dataset.tairitsuReady==="hydrated";window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:JSON.stringify({{ready:w&&h,wasm_loaded:w,hydrated:h,url:location.href}})}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<ReadyResponse>(&data) {
                                            Ok(rr) => { let _ = r.send(Ok(rr)); }
                                            Err(_) => { let _ = r.send(Err(format!("ready parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::Press { key, modifiers, count, resp } => {
                            let mod_str = modifiers.iter().map(|m| format!("{:?}:true", m.to_lowercase())).collect::<Vec<_>>().join(",");
                            let js = format!(r#"(()=>{{for(var i=0;i<{count};i++)document.dispatchEvent(new KeyboardEvent('keydown',{{key:{key:?},code:{key:?},{mod_str},bubbles:true}}));for(var i=0;i<{count};i++)document.dispatchEvent(new KeyboardEvent('keyup',{{key:{key:?},code:{key:?},{mod_str},bubbles:true}}))}})()"#);
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(50));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::Scroll { selector, x, y, resp } => {
                            let js = if let Some(sel) = &selector {
                                format!(r#"(()=>{{var el=document.querySelector({:?});if(el)el.scrollBy({x},{y})}})()"#, sel)
                            } else {
                                format!(r#"window.scrollBy({},{})"#, x, y)
                            };
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(100));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::Resize { width, height, resp } => {
                            let _ = webview.evaluate_script(&format!("window.resizeTo({},{})", width, height));
                            std::thread::sleep(Duration::from_millis(200));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::Viewport { resp } => {
                            let id = next_id; next_id += 1;
                            let js = format!(r#"(()=>{{try{{var dpr=window.devicePixelRatio||1;window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:JSON.stringify({{width:window.innerWidth,height:window.innerHeight,device_pixel_ratio:dpr}})}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<ViewportResponse>(&data) {
                                            Ok(v) => { let _ = r.send(Ok(v)); }
                                            Err(_) => { let _ = r.send(Err(format!("viewport parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::Drag { from_selector, to_selector, steps, resp } => {
                            let js = format!(r#"(()=>{{try{{var src=document.querySelector({from:?});var dst=document.querySelector({to:?});if(!src||!dst){{window.ipc.postMessage(JSON.stringify({{id:-1,ok:false,error:'element not found'}}));return}}var sr=src.getBoundingClientRect();var dr=dst.getBoundingClientRect();var sx=sr.x+sr.width/2,sy=sr.y+sr.height/2;var dx=dr.x+dr.width/2,dy=dr.y+dr.height/2;src.dispatchEvent(new MouseEvent('mousedown',{{clientX:sx,clientY:sy,bubbles:true}}));for(var i=1;i<={steps};i++){{var t=i/{steps};var cx=sx+(dx-sx)*t,cy=sy+(dy-sy)*t;document.dispatchEvent(new MouseEvent('mousemove',{{clientX:cx,clientY:cy,bubbles:true}}))}}dst.dispatchEvent(new MouseEvent('mouseup',{{clientX:dx,clientY:dy,bubbles:true}}));dst.dispatchEvent(new MouseEvent('drop',{{clientX:dx,clientY:dy,bubbles:true}}))}}catch(e){{}}}})()"#,
                                from = from_selector, to = to_selector, steps = steps);
                            let _ = webview.evaluate_script(&js);
                            std::thread::sleep(Duration::from_millis(200));
                            let _ = resp.send(Ok(()));
                        }
                        BrowserCommand::A11y { selector, depth, resp } => {
                            let id = next_id; next_id += 1;
                            let sel_js = match &selector {
                                Some(s) => format!("document.querySelector({:?})", s),
                                None => "document.body".to_string(),
                            };
                            let js = build_a11y_js(id, &sel_js, depth);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<Vec<A11yNode>>(&data) {
                                            Ok(nodes) => { let _ = r.send(Ok(nodes)); }
                                            Err(_) => { let _ = r.send(Err(format!("a11y parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::Network { resp } => {
                            let id = next_id; next_id += 1;
                            let js = format!(r#"(()=>{{try{{var entries=performance.getEntriesByType('resource').slice(0,100).map(function(e){{return{{name:e.name,type:e.initiatorType||'unknown',duration:Math.round(e.duration*100)/100,size:e.transferSize||0,url:e.name}}}});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:JSON.stringify({{resources:entries}})}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<NetworkResponse>(&data) {
                                            Ok(n) => { let _ = r.send(Ok(n)); }
                                            Err(_) => { let _ = r.send(Err(format!("network parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::Performance { resp } => {
                            let id = next_id; next_id += 1;
                            let js = build_performance_js(id);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<PerformanceMetrics>(&data) {
                                            Ok(p) => { let _ = r.send(Ok(p)); }
                                            Err(_) => { let _ = r.send(Err(format!("perf parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::WebSocket { resp } => {
                            let id = next_id; next_id += 1;
                            let js = build_websocket_js(id);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => {
                                        match serde_json::from_str::<WebSocketInfo>(&data) {
                                            Ok(w) => { let _ = r.send(Ok(w)); }
                                            Err(_) => { let _ = r.send(Err(format!("ws parse: {}", data))); }
                                        }
                                    }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
                        }
                        BrowserCommand::Shutdown => {
                            *connected.blocking_write() = false;
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                }
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    *connected.blocking_write() = false;
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }

    fn parse_eval_result(data: &str) -> (serde_json::Value, &'static str) {
        if data == "null" || data.is_empty() {
            return (serde_json::Value::Null, "null");
        }
        if data.starts_with('"') {
            return (serde_json::Value::String(data.trim_matches('"').to_string()), "string");
        }
        if data == "true" || data == "false" {
            return (serde_json::Value::Bool(data == "true"), "boolean");
        }
        if let Ok(n) = data.parse::<f64>() {
            return (serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0))), "number");
        }
        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(v) => (v, "object"),
            Err(_) => (serde_json::Value::String(data.to_string()), "string"),
        }
    }

    #[cfg(target_os = "linux")]
    fn capture_webkit_snapshot(webview: &wry::WebView) -> Result<(Vec<u8>, u32, u32), String> {
        use wry::WebViewExtUnix;
        use webkit2gtk::{SnapshotRegion, SnapshotOptions, WebViewExt};
        use std::sync::mpsc;

        let wk = webview.webview();
        let (tx, rx) = mpsc::channel();
        wk.snapshot(
            SnapshotRegion::Visible,
            SnapshotOptions::NONE,
            None::<&gtk::gio::Cancellable>,
            move |result| { let _ = tx.send(result); },
        );

        let deadline = Instant::now() + Duration::from_secs(5);
        let surface = loop {
            if let Ok(r) = rx.try_recv() {
                break r.map_err(|e| format!("snapshot: {}", e))?;
            }
            if Instant::now() > deadline {
                return Err("snapshot timed out".into());
            }
            gtk::main_iteration_do(false);
            std::thread::sleep(Duration::from_millis(10));
        };

        let mut png = Vec::new();
        {
            let s = &surface;
            let w = unsafe { cairo_sys::cairo_image_surface_get_width(s.to_raw_none()) as u32 };
            let h = unsafe { cairo_sys::cairo_image_surface_get_height(s.to_raw_none()) as u32 };
            let stride = unsafe { cairo_sys::cairo_image_surface_get_stride(s.to_raw_none()) as usize };
            let data_ptr = unsafe { cairo_sys::cairo_image_surface_get_data(s.to_raw_none()) };
            if w == 0 || h == 0 || data_ptr.is_null() {
                return Err("empty cairo surface".into());
            }
            let data = unsafe { std::slice::from_raw_parts(data_ptr, h as usize * stride) };
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for row in 0..h as usize {
                for col in 0..w as usize {
                    let off = row * stride + col * 4;
                    if off + 4 > data.len() { break; }
                    let px = &data[off..off + 4];
                    rgba.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
                }
            }
            let buffer = image::RgbaImage::from_raw(w, h, rgba).ok_or("invalid image dims")?;
            buffer.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
                .map_err(|e| format!("png encode: {}", e))?;
        }
        if png.is_empty() {
            return Err("empty PNG output".into());
        }

        let (w, h) = {
            let img = image::load_from_memory(&png)
                .map_err(|e| format!("parse png: {}", e))?;
            (img.width(), img.height())
        };
        Ok((png, w, h))
    }

    #[cfg(not(target_os = "linux"))]
    fn capture_webkit_snapshot(_webview: &wry::WebView) -> Result<(Vec<u8>, u32, u32), String> {
        Err("pixel capture not supported on this platform".into())
    }

    fn build_screenshot_eval_js(selector: Option<&str>, full_page: bool) -> String {
        let h_expr = if full_page { "Math.max(document.documentElement.scrollHeight,window.innerHeight)" } else { "window.innerHeight" };
        let capture_logic = if selector.is_some() {
            "ctx.fillRect(0,0,w,h)".to_string()
        } else {
            r#"ctx.fillStyle='#fff';ctx.fillRect(0,0,w,h);var allEls=document.body.querySelectorAll('*');var els=[];for(var i=0;i<allEls.length;i++){var er=allEls[i].getBoundingClientRect();if(er.width<1||er.height<1||er.bottom<0||er.right<0||er.top>h||er.left>w)continue;els.push(allEls[i])}els.forEach(function(e){var er=e.getBoundingClientRect();var cs=getComputedStyle(e);ctx.fillStyle=cs.backgroundColor;if(ctx.fillStyle!=='rgba(0, 0, 0, 0)'&&ctx.fillStyle!=='transparent'){ctx.fillRect(er.x,er.y,er.width,er.height)}ctx.fillStyle=cs.borderColor;if(ctx.fillStyle!=='rgba(0, 0, 0, 0)'&&ctx.fillStyle!=='transparent'){var bw=parseFloat(cs.borderWidth)||1;if(bw>0){ctx.fillRect(er.x,er.y,er.width,bw);ctx.fillRect(er.x,er.y+er.height-bw,er.width,bw);ctx.fillRect(er.x,er.y,bw,er.height);ctx.fillRect(er.x+er.width-bw,er.y,bw,er.height)}}if(e.shadowRoot)return;if(e.childNodes.length===1&&e.childNodes[0].nodeType===3){var txt=(e.childNodes[0].textContent||'').trim();if(txt&&txt.length<200){var fs=parseFloat(cs.fontSize)||16;ctx.font=cs.fontWeight+' '+fs+'px '+(cs.fontFamily||'sans-serif');ctx.fillStyle=cs.color||'#000';ctx.textBaseline='top';var maxW=er.width-4;if(maxW>0)ctx.fillText(txt.substring(0,100),er.x+2,er.y+2,maxW)}}})"#.to_string()
        };
        format!(
            r#"function(){{var w=Math.max(document.documentElement.clientWidth,window.innerWidth||1280);var h={h_expr}||720;var c=document.createElement('canvas');c.width=w;c.height=h;var ctx=c.getContext('2d');{capture_logic};return c.toDataURL('image/png').split(',')[1]}}()"#
        )
    }

    fn build_a11y_js(id: i32, sel_js: &str, depth: u32) -> String {
        let js_body = r#"
(function(){
try{
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
}catch(e){throw e}
})()"#;
        let eval_js = js_body
            .replace("SEL_JS", sel_js)
            .replace("DEPTH", &depth.to_string());
        format!(
            r#"(()=>{{try{{var r=({eval_js});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:r}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
        )
    }

    #[allow(dead_code)]
    fn build_screenshot_js(id: i32, selector: Option<&str>, full_page: bool) -> String {
        let h_expr = if full_page { "Math.max(document.documentElement.scrollHeight,window.innerHeight)" } else { "window.innerHeight" };
        let capture_logic = if let Some(sel) = selector {
            format!(
                r#"var el=document.querySelector({:?});if(!el){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:'element not found'}}));return}}var r=el.getBoundingClientRect();c.width=Math.ceil(r.width*dpr);c.height=Math.ceil(r.height*dpr);ctx.scale(dpr,dpr);ctx.translate(-r.x,-r.y);document.querySelectorAll('*').forEach(function(e){{var er=e.getBoundingClientRect();ctx.fillStyle=getComputedStyle(e).backgroundColor;if(er.width>0&&er.height>0)ctx.fillRect(er.x,er.y,er.width,er.height)}})"#,
                sel
            )
        } else {
            format!(
                r#"c.width=w*dpr;c.height=h*dpr;ctx.scale(dpr,dpr);ctx.fillStyle='#fff';ctx.fillRect(0,0,w,h);Array.from(document.body.querySelectorAll('*')).slice(0,500).forEach(function(e){{var er=e.getBoundingClientRect();if(er.width<1||er.height<1||er.bottom<0||er.right<0||er.top>h||er.left>w)return;var s=getComputedStyle(e);ctx.fillStyle=s.backgroundColor||'transparent';ctx.fillRect(er.x,er.y,er.width,er.height)}});Array.from(document.body.querySelectorAll('*')).slice(0,500).forEach(function(e){{var er=e.getBoundingClientRect();var s=getComputedStyle(e);if(er.width<1||er.height<1)return;if(e.childNodes.length===1&&e.childNodes[0].nodeType===3){{ctx.font=(s.fontSize||'16px')+' '+(s.fontFamily||'sans-serif');ctx.fillStyle=s.color||'#000';ctx.textBaseline='top';var txt=(e.childNodes[0].textContent||'').trim();if(txt)ctx.fillText(txt.substring(0,Math.floor(er.width/8)),er.x,er.y)}}}})"#
            )
        };
        format!(
            r#"(()=>{{try{{var w=Math.max(document.documentElement.clientWidth,window.innerWidth||1280);var h={h_expr}||720;var dpr=window.devicePixelRatio||1;var c=document.createElement('canvas');var ctx=c.getContext('2d');{capture_logic}var base64=c.toDataURL('image/png').split(',')[1];window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:base64}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
        )
    }

    fn build_performance_js(id: i32) -> String {
        format!(
            r#"(()=>{{try{{var nav=performance.getEntriesByType('navigation')[0]||{{}};var fcp=null;try{{fcp=performance.getEntriesByName('first-contentful-paint')[0].startTime||null}}catch(e){{}}var dn=document.querySelectorAll('*').length;var heap=null;try{{heap=Math.round((performance.memory?performance.memory.usedJSHeapSize:0)/1048576*100)/100}}catch(e){{}}var d={{dom_content_loaded_ms:Math.round((nav.domContentLoadedEventEnd-nav.startTime)*100)/100||null,dom_complete_ms:Math.round((nav.domComplete-nav.startTime)*100)/100||null,load_event_ms:Math.round((nav.loadEventEnd-nav.startTime)*100)/100||null,fcp_ms:fcp?Math.round(fcp*100)/100:null,lcp_ms:null,cls:null,dom_nodes:dn,js_heap_used_mb:heap,wasm_loaded:!!globalThis.__wasmExports,hydrated:document.documentElement.dataset.tairitsuReady==='hydrated',timestamp:new Date().toISOString()}};window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:JSON.stringify(d)}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
        )
    }

    fn build_websocket_js(id: i32) -> String {
        format!(
            r#"(()=>{{try{{var c=0;var conns=[];var t=window._wsTracker||[];t.forEach(function(ws){{c++;conns.push({{url:ws.url||'unknown',state:ws.readyState===0?'connecting':ws.readyState===1?'open':ws.readyState===2?'closing':'closed',created_at_ms:null}})}});window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:JSON.stringify({{active_count:c,connections:conns}})}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
        )
    }
}

// ── DebugState ────────────────────────────────────────────────────────────

#[derive(Clone)]
struct DebugState {
    config: Config, dev_port: u16, debug_port: u16, start_time: Instant,
    base_url: String, console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
    errors: Arc<RwLock<Vec<ErrorEntry>>>,
    rejections: Arc<RwLock<Vec<ErrorEntry>>>,
    browser: Option<Arc<BrowserHandle>>, browser_engine: String,
}

impl DebugState {
    #[allow(dead_code)]
    fn new(config: Config, dev_port: u16, debug_port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", dev_port),
            config, dev_port, debug_port, start_time: Instant::now(),
            console_log: Arc::new(RwLock::new(Vec::new())),
            errors: Arc::new(RwLock::new(Vec::new())),
            rejections: Arc::new(RwLock::new(Vec::new())),
            browser: None, browser_engine: "none".into(),
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

    #[cfg(target_os = "linux")]
    {
        if std::env::var("DISPLAY").map(|d| d.is_empty()).unwrap_or(true) {
            crate::log_info!("[debug-headless] DISPLAY not set, auto-detecting Xvfb...");
            let check = std::process::Command::new("xdpyinfo")
                .env("DISPLAY", ":99")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            if check.map(|s| s.success()).unwrap_or(false) {
                unsafe { std::env::set_var("DISPLAY", ":99"); }
                crate::log_ok!("[debug-headless] Using DISPLAY=:99 (Xvfb detected)");
            } else {
                crate::log_info!("[debug-headless] No Xvfb on :99, attempting to start...");
                let started = std::process::Command::new("Xvfb")
                    .args([":99", "-screen", "0", "1920x1080x24", "-ac", "-nolisten", "tcp"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .map(|mut c| {
                        std::thread::sleep(Duration::from_millis(500));
                        c.try_wait().ok().flatten().map(|s| s.success()).unwrap_or(true)
                    })
                    .unwrap_or(false);
                if started {
                    unsafe { std::env::set_var("DISPLAY", ":99"); }
                    crate::log_ok!("[debug-headless] Xvfb started on :99");
                } else {
                    crate::log_fail!("[debug-headless] Failed to start Xvfb. Install: apt install xvfb");
                }
            }
        }
    }

    let browser = engine::spawn_browser(base_url.clone(), None, console_log.clone())
        .ok().map(Arc::new);

    let browser_engine = if browser.is_some() {
        #[cfg(feature = "debug-browser")] { "wry" }
        #[cfg(not(feature = "debug-browser"))] { "none" }
    } else { "none" }.to_string();

    let state = DebugState {
        config: config.clone(), dev_port, debug_port,
        base_url, console_log, errors: Arc::new(RwLock::new(Vec::new())),
        rejections: Arc::new(RwLock::new(Vec::new())),
        browser, browser_engine, start_time: Instant::now(),
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
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    crate::log_ok!("Debug API v{} listening on http://localhost:{}", DEBUG_API_VERSION, debug_port);
    crate::log_info!("Endpoints: /health /info /ready /navigate /screenshot /click /type /press /scroll /evaluate /console /dom /dom/computed /viewport /resize /errors /drag /a11y /batch /network /performance /websocket /source-map");

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
        pid: std::process::id(), started_at_iso: chrono::Utc::now().to_rfc3339(),
        uptime_secs: state.uptime_secs(), browser_connected: bc,
        browser_engine: state.browser_engine.clone(), viewport: [DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H],
    }))
}

async fn ready_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<ReadyResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::IsReady { resp: tx }).await.is_err() { return chan_closed::<ReadyResponse>(); }
    await_op(rx).await
}

async fn navigate_handler(State(state): State<DebugState>, Json(req): Json<NavigateRequest>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<NavigateResponse>() };
    let target = if req.url.starts_with("http") { req.url } else { format!("{}{}", state.base_url, req.url) };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Navigate { url: target, wait_for: req.wait_for, resp: tx }).await.is_err() { return chan_closed::<NavigateResponse>(); }
    await_op(rx).await
}

async fn screenshot_handler(State(state): State<DebugState>, Json(params): Json<ScreenshotParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<ScreenshotResponse>() };
    let (tx, rx) = oneshot::channel();
    let mode = params.mode.clone().unwrap_or_default();
    if br.send(BrowserCommand::Screenshot { selector: params.selector, full_page: params.full_page.unwrap_or(false), mode, resp: tx }).await.is_err() { return chan_closed::<ScreenshotResponse>(); }
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

async fn press_handler(State(state): State<DebugState>, Json(req): Json<PressRequest>) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Press { key: req.key, modifiers: req.modifiers.unwrap_or_default(), count: req.count.unwrap_or(1), resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn scroll_handler(State(state): State<DebugState>, Json(req): Json<ScrollRequest>) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (tx, rx) = oneshot::channel();
        let (x, y) = match req.direction.as_deref() {
            Some("up") => (0.0, -(req.amount.unwrap_or(300.0))),
            Some("down") => (0.0, req.amount.unwrap_or(300.0)),
            Some("left") => (-(req.amount.unwrap_or(300.0)), 0.0),
            Some("right") => (req.amount.unwrap_or(300.0), 0.0),
            _ => (req.x.unwrap_or(0.0), req.y.unwrap_or(0.0)),
        };
    if br.send(BrowserCommand::Scroll { selector: req.selector, x, y, resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn evaluate_handler(State(state): State<DebugState>, Json(req): Json<EvaluateRequest>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<EvaluateResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Evaluate { expression: req.expression, await_promise: req.await_promise.unwrap_or(false), resp: tx }).await.is_err() { return chan_closed::<EvaluateResponse>(); }
    await_op(rx).await
}

async fn console_handler(State(state): State<DebugState>, Query(params): Query<ConsoleQueryParams>) -> impl IntoResponse {
    let entries = state.console_log.read().await;
    let mut filtered: Vec<ConsoleEntry> = entries.iter().filter(|e| {
        if let Some(ref levels) = params.level {
            let allowed: Vec<&str> = levels.split(',').collect();
            if !allowed.contains(&e.level.as_str()) { return false; }
        }
        if let Some(ref src) = params.source {
            if e.source.as_deref() != Some(src.as_str()) { return false; }
        }
        true
    }).cloned().collect();
    if let Some(limit) = params.limit { filtered.truncate(limit); }
    ResponseJson(ApiResponse::ok(ConsoleResponse { entries: filtered }))
}

async fn console_clear_handler(State(state): State<DebugState>) -> impl IntoResponse {
    state.console_log.write().await.clear();
    ResponseJson(ApiResponse::ok(serde_json::json!({"cleared": true})))
}

async fn dom_query_handler(State(state): State<DebugState>, Query(params): Query<DomQueryParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<DomNodeResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::DomQuery { selector: params.selector, attribute: params.attribute, computed: None, resp: tx }).await.is_err() { return chan_closed::<DomNodeResponse>(); }
    await_op(rx).await
}

async fn computed_style_handler(State(state): State<DebugState>, Json(params): Json<ComputedStyleParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<ComputedStyleResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::DomQuery { selector: params.selector.clone(), attribute: None, computed: params.properties, resp: tx }).await.is_err() { return chan_closed::<ComputedStyleResponse>(); }
    match tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await {
        Ok(Ok(Ok(dom))) => {
            let computed = dom.computed.unwrap_or_default();
            (StatusCode::OK, ResponseJson(ApiResponse::ok(ComputedStyleResponse { selector: params.selector, properties: computed })))
        }
        Ok(Ok(Err(e))) => (StatusCode::BAD_REQUEST, ResponseJson(ApiResponse::err(e))),
        Ok(Err(_)) => chan_closed::<ComputedStyleResponse>(),
        Err(_) => (StatusCode::GATEWAY_TIMEOUT, ResponseJson(ApiResponse::err("Operation timed out"))),
    }
}

async fn viewport_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<ViewportResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Viewport { resp: tx }).await.is_err() { return chan_closed::<ViewportResponse>(); }
    await_op(rx).await
}

async fn resize_handler(State(state): State<DebugState>, Json(req): Json<ResizeRequest>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (w, h) = match req.preset.as_deref() {
        Some("mobile") => (375, 812),
        Some("tablet") => (768, 1024),
        Some("desktop") => (1280, 720),
        Some("wide") => (1920, 1080),
        _ => (req.width.unwrap_or(DEFAULT_VIEWPORT_W), req.height.unwrap_or(DEFAULT_VIEWPORT_H)),
    };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Resize { width: w, height: h, resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn errors_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(ErrorsResponse {
        errors: state.errors.read().await.clone(),
        unhandled_rejections: state.rejections.read().await.clone(),
    }))
}

async fn drag_handler(State(state): State<DebugState>, Json(req): Json<DragRequest>) -> (StatusCode, ResponseJson<ApiResponse<()>>) {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<()>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Drag { from_selector: req.from_selector, to_selector: req.to_selector, steps: req.steps.unwrap_or(10), resp: tx }).await.is_err() { return chan_closed::<()>(); }
    await_op(rx).await
}

async fn a11y_handler(State(state): State<DebugState>, Query(params): Query<A11yQueryParams>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<Vec<A11yNode>>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::A11y { selector: params.selector, depth: params.depth.unwrap_or(5), resp: tx }).await.is_err() { return chan_closed::<Vec<A11yNode>>(); }
    await_op(rx).await
}

async fn batch_handler(State(state): State<DebugState>, Json(req): Json<BatchRequest>) -> impl IntoResponse {
    let mut results = Vec::with_capacity(req.operations.len());
    for (i, op) in req.operations.into_iter().enumerate() {
        let start = Instant::now();
        let name = match &op {
            BatchOperation::Screenshot { name, .. } => name.clone().unwrap_or_else(|| format!("screenshot_{}", i)),
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
        }.to_string();

        let (success, data, error) = match execute_batch_op(&state, op).await {
            Ok(d) => (true, Some(d), None),
            Err(e) => (false, None, Some(e)),
        };
        results.push(BatchResult {
            name, op_type, success, data, error,
            duration_ms: start.elapsed().as_millis() as u64,
        });
    }
    ResponseJson(ApiResponse::ok(serde_json::json!({ "results": results })))
}

async fn execute_batch_op(state: &DebugState, op: BatchOperation) -> Result<serde_json::Value, String> {
    let br = state.browser.as_ref().ok_or("No browser")?;
    match op {
        BatchOperation::Navigate { url, wait_for } => {
            let target = if url.starts_with("http") { url } else { format!("{}{}", state.base_url, url) };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Navigate { url: target, wait_for, resp: tx }).await.map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?;
            r.map(|nav| serde_json::to_value(nav).unwrap_or_default())
        }
        BatchOperation::Screenshot { selector, full_page, mode, .. } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Screenshot { selector, full_page: full_page.unwrap_or(false), mode: mode.unwrap_or_default(), resp: tx }).await.map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?;
            r.map(|ss| serde_json::json!({ "mode": ss.mode, "width": ss.width, "height": ss.height, "data_len": ss.data.len() }))
        }
        BatchOperation::Click { selector } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Click { selector, resp: tx }).await.map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?.map_err(|e| e)?;
            Ok(serde_json::json!({ "clicked": true }))
        }
        BatchOperation::Evaluate { expression } => {
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Evaluate { expression, await_promise: false, resp: tx }).await.map_err(|e| e.to_string())?;
            let r = tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?;
            r.map(|ev| serde_json::json!({ "result": ev.result, "type": ev.r#type }))
        }
        BatchOperation::Wait { ms } => {
            tokio::time::sleep(Duration::from_millis(ms)).await;
            Ok(serde_json::json!({ "waited_ms": ms }))
        }
        BatchOperation::Scroll { selector, direction, amount } => {
            let (x, y) = match direction.as_deref() {
                Some("up") => (0.0, -(amount.unwrap_or(300.0))),
                Some("down") => (0.0, amount.unwrap_or(300.0)),
                Some("left") => (-(amount.unwrap_or(300.0)), 0.0),
                Some("right") => (amount.unwrap_or(300.0), 0.0),
                _ => (0.0, amount.unwrap_or(300.0)),
            };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Scroll { selector, x, y, resp: tx }).await.map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?.map_err(|e| e)?;
            Ok(serde_json::json!({ "scrolled": true }))
        }
        BatchOperation::Resize { width, height, preset } => {
            let (w, h) = match preset.as_deref() {
                Some("mobile") => (375, 812),
                Some("tablet") => (768, 1024),
                Some("desktop") => (1280, 720),
                Some("wide") => (1920, 1080),
                _ => (width.unwrap_or(DEFAULT_VIEWPORT_W), height.unwrap_or(DEFAULT_VIEWPORT_H)),
            };
            let (tx, rx) = oneshot::channel();
            br.send(BrowserCommand::Resize { width: w, height: h, resp: tx }).await.map_err(|e| e.to_string())?;
            tokio::time::timeout(Duration::from_secs(OP_TIMEOUT_SECS), rx).await.map_err(|_| "timeout".to_string())?.map_err(|_| "channel closed".to_string())?.map_err(|e| e)?;
            Ok(serde_json::json!({ "resized": [w, h] }))
        }
    }
}

async fn network_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<NetworkResponse>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Network { resp: tx }).await.is_err() { return chan_closed::<NetworkResponse>(); }
    await_op(rx).await
}

async fn performance_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<PerformanceMetrics>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::Performance { resp: tx }).await.is_err() { return chan_closed::<PerformanceMetrics>(); }
    await_op(rx).await
}

async fn websocket_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let br = match &state.browser { Some(b) => b, None => return svc_unavailable::<WebSocketInfo>() };
    let (tx, rx) = oneshot::channel();
    if br.send(BrowserCommand::WebSocket { resp: tx }).await.is_err() { return chan_closed::<WebSocketInfo>(); }
    await_op(rx).await
}

async fn source_map_handler(Json(req): Json<SourceMapRequest>) -> impl IntoResponse {
    let frames = parse_wasm_stack(&req.stack);
    ResponseJson(ApiResponse::ok(SourceMapResponse { frames, raw: req.stack }))
}

fn parse_wasm_stack(stack: &str) -> Vec<StackFrame> {
    let mut frames = Vec::new();
    for line in stack.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
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
        frames.push(StackFrame { file, line: line_num, col, func, raw });
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
