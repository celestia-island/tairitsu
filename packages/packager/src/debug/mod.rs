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

use crate::config::Config;

const DEBUG_API_VERSION: &str = "0.2.0";
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
struct ScreenshotParams { selector: Option<String>, full_page: Option<bool>, format: Option<String>, mode: Option<String> }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenshotResponse { data: String, mime_type: String, width: u32, height: u32, mode: String }

#[derive(Debug, Clone, Deserialize)]
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
        use tao::rwh_06::HasWindowHandle;
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

        let x11_window_id: Option<u32> = window.window_handle().ok().and_then(|h| {
            match h.as_raw() {
                tao::rwh_06::RawWindowHandle::Xlib(x) => Some(x.window as u32),
                _ => None,
            }
        });

        let pending: PendingMap = StdArc::new(Mutex::new(HashMap::new()));
        let pending_ipc = pending.clone();

        crate::log_info!("[wry] Creating WebView with URL {}...", base_url);
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
                            let wait_ms = match wait_for.as_deref() {
                                Some("hydration") | Some("ready") => 1500,
                                Some("load") => 1000,
                                _ => 500,
                            };
                            std::thread::sleep(Duration::from_millis(wait_ms));
                            if matches!(wait_for.as_deref(), Some("hydration") | Some("ready")) {
                                for _ in 0..20 {
                                    std::thread::sleep(Duration::from_millis(200));
                                    let check_id = next_id; next_id += 1;
                                    let check_js = format!(r#"(()=>{{try{{var r=document.documentElement.dataset.tairitsuReady==='hydrated';window.ipc.postMessage(JSON.stringify({{id:{check_id},ok:true,data:String(r)}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{check_id},ok:false,error:e.message}}))}}}})()"#);
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    pending.lock().unwrap().insert(check_id, Box::new(move |result| { let _ = tx.send(result); }));
                                    let _ = webview.evaluate_script(&check_js);
                                    match rx.recv_timeout(Duration::from_secs(1)) {
                                        Ok(Ok(data)) if data == "true" => break,
                                        _ => continue,
                                    }
                                }
                            }
                            let _ = resp.send(Ok(NavigateResponse { url: target, title: String::new() }));
                        }
                        BrowserCommand::Screenshot { selector, full_page, mode, resp } => {
                            if mode == "pixel" {
                                if let Some(win_id) = x11_window_id {
                                    match capture_x11_window(win_id) {
                                        Ok((png_bytes, w, h)) => {
                                            let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
                                            let _ = resp.send(Ok(ScreenshotResponse { data: b64, mime_type: "image/png".into(), width: w, height: h, mode: "pixel".into() }));
                                        }
                                        Err(e) => { let _ = resp.send(Err(format!("pixel capture: {}", e))); }
                                    }
                                    return;
                                }
                            }
                            let id = next_id; next_id += 1;
                            let js = build_screenshot_js(id, selector.as_deref(), full_page);
                            let r = resp;
                            pending.lock().unwrap().insert(id, Box::new(move |result| {
                                match result {
                                    Ok(data) => { let _ = r.send(Ok(ScreenshotResponse { data, mime_type: "image/png".into(), width: DEFAULT_VIEWPORT_W, height: DEFAULT_VIEWPORT_H, mode: "canvas".into() })); }
                                    Err(e) => { let _ = r.send(Err(e)); }
                                }
                            }));
                            let _ = webview.evaluate_script(&js);
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

    fn capture_x11_window(window_id: u32) -> Result<(Vec<u8>, u32, u32), String> {
        #[cfg(target_os = "linux")]
        {
            use base64::Engine;
            let (conn, _) = x11rb::connect(None).map_err(|e| format!("x11 connect: {}", e))?;
            let geom = x11rb::protocol::xproto::get_geometry(&conn, window_id)
                .map_err(|e| format!("get_geometry: {}", e))?
                .reply()
                .map_err(|e| format!("geom reply: {}", e))?;
            let w = geom.width as u32;
            let h = geom.height as u32;
            let img = x11rb::protocol::xproto::get_image(
                &conn, x11rb::protocol::xproto::ImageFormat::Z_PIXMAP,
                window_id, 0, 0, geom.width, geom.height, u32::MAX,
            ).map_err(|e| format!("get_image: {}", e))?
            .reply().map_err(|e| format!("image reply: {}", e))?;

            let bpp = if img.depth <= 8 { 1 } else if img.depth <= 16 { 2 } else if img.depth <= 24 { 3 } else { 4 };
            let mut rgba = Vec::with_capacity((w * h * 4) as usize);
            for chunk in img.data.chunks(bpp as usize) {
                match bpp {
                    4 => { rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], 255]); }
                    3 => { rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], 255]); }
                    2 => { rgba.extend_from_slice(&[chunk[1], chunk[0], 0, 255]); }
                    _ => { rgba.extend_from_slice(&[chunk[0], chunk[0], chunk[0], 255]); }
                }
            }
            let buffer = image::RgbaImage::from_raw(w, h, rgba).ok_or("invalid image dims")?;
            let mut png = Vec::new();
            buffer.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| format!("png encode: {}", e))?;
            Ok((png, w, h))
        }
        #[cfg(not(target_os = "linux"))]
        { Err("pixel capture not supported on this platform".into()) }
    }

    fn build_screenshot_js(id: i32, selector: Option<&str>, full_page: bool) -> String {
        let h_expr = if full_page { "Math.max(document.documentElement.scrollHeight,window.innerHeight)" } else { "window.innerHeight" };
        let capture_logic = if let Some(sel) = selector {
            format!(
                r#"var el=document.querySelector({:?});if(!el){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:'element not found'}}));return}}var r=el.getBoundingClientRect();c.width=Math.ceil(r.width*dpr);c.height=Math.ceil(r.height*dpr);ctx.scale(dpr,dpr);ctx.translate(-r.x,-r.y);document.querySelectorAll('*').forEach(function(e){{var er=e.getBoundingClientRect();ctx.fillStyle=getComputedStyle(e).backgroundColor;if(er.width>0&&er.height>0)ctx.fillRect(er.x,er.y,er.width,er.height)}})"#,
                sel
            )
        } else {
            format!(
                r#"c.width=w*dpr;c.height=h*dpr;ctx.scale(dpr,dpr);document.querySelectorAll('*').forEach(function(e){{var er=e.getBoundingClientRect();ctx.fillStyle=getComputedStyle(e).backgroundColor||'transparent';if(er.width>0&&er.height>0&&er.bottom>0&&er.right>0&&er.top<h&&er.left<w)ctx.fillRect(er.x,er.y,er.width,er.height)}});document.querySelectorAll('*').forEach(function(e){{var er=e.getBoundingClientRect();var s=getComputedStyle(e);if(e.childNodes.length===1&&e.childNodes[0].nodeType===3&&er.width>0&&er.height>0){{ctx.font=s.fontSize+' '+(s.fontFamily||'sans-serif');ctx.fillStyle=s.color||'inherit';ctx.textBaseline='top';var txt=e.childNodes[0].textContent.trim();if(txt)ctx.fillText(txt.substring(0,Math.floor(er.width/8)),er.x,er.y+parseInt(s.paddingTop||0))}}}})"#
            )
        };
        format!(
            r#"(()=>{{try{{var w=Math.max(document.documentElement.clientWidth,window.innerWidth||1280);var h={h_expr}||720;var dpr=window.devicePixelRatio||1;var c=document.createElement('canvas');var ctx=c.getContext('2d');{capture_logic}var base64=c.toDataURL('image/png').split(',')[1];window.ipc.postMessage(JSON.stringify({{id:{id},ok:true,data:base64}}))}}catch(e){{window.ipc.postMessage(JSON.stringify({{id:{id},ok:false,error:e.message}}))}}}})()"#
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
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    crate::log_ok!("Debug API v{} listening on http://localhost:{}", DEBUG_API_VERSION, debug_port);
    crate::log_info!("Endpoints: /health /info /ready /navigate /screenshot /click /type /press /scroll /evaluate /console /dom /dom/computed /viewport /resize /errors");

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
    let props = params.properties.clone();
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
