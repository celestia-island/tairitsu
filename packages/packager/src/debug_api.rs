//! Debug API for browser automation — exposed by the dev server daemon.
//!
//! Uses **wry** (headless/offscreen WebView) so:
//!   - No visible window pops up
//!   - Cross-platform (WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux)

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, mpsc};

// ─────────────────────────────────────────────────────
// Command enum — sent from axum handlers to wry thread
// ─────────────────────────────────────────────────────

#[allow(dead_code)]
enum ComCommand {
    Navigate {
        url: String,
    },
    EvaluateScript {
        script: String,
        response_tx: Option<mpsc::Sender<String>>,
    },
    Click {
        selector: String,
    },
    Screenshot {
        response_tx: Option<mpsc::Sender<Result<String, String>>>,
    },
    Shutdown,
}

// ─────────────────────────────────────────────────────
// DebugApiState — Send+Sync safe wrapper around wry thread
// ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct DebugApiState {
    tx: mpsc::Sender<ComCommand>,
    port: std::sync::Arc<std::sync::Mutex<u16>>,
}

impl DebugApiState {
    pub fn new(port: u16) -> Self {
        let (tx, rx) = mpsc::channel::<ComCommand>();
        let port_arc = std::sync::Arc::new(std::sync::Mutex::new(port));
        std::thread::spawn(move || {
            if let Err(e) = wry_thread_main(rx) {
                eprintln!("[debug-api] wry thread error: {}", e);
            }
        });
        Self { tx, port: port_arc }
    }

    pub async fn set_port(&self, port: u16) {
        if let Ok(mut p) = self.port.lock() {
            *p = port;
        }
    }

    pub async fn launch_browser(&self, url: &str) -> crate::Result<()> {
        let _ = self.tx.send(ComCommand::Navigate {
            url: url.to_string(),
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        crate::log_ok!("Debug browser launched (headless wry): {}", url);
        Ok(())
    }

    pub async fn is_browser_connected(&self) -> bool {
        true
    }

    pub async fn port(&self) -> u16 {
        *self.port.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn send_cmd(&self, cmd: ComCommand) {
        let _ = self.tx.send(cmd);
    }

    fn send_eval(&self, script: &str) -> Option<String> {
        let (tx, rx) = mpsc::channel();
        if self
            .tx
            .send(ComCommand::EvaluateScript {
                script: script.to_string(),
                response_tx: Some(tx),
            })
            .is_ok()
        {
            rx.recv_timeout(std::time::Duration::from_secs(10)).ok()
        } else {
            None
        }
    }
}

// ─────────────────────────────────────────────────────
// Wry thread — owns hidden tao window + wry WebView
// ─────────────────────────────────────────────────────

fn wry_thread_main(rx: mpsc::Receiver<ComCommand>) -> Result<(), String> {
    use tao::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    use wry::WebViewBuilder;

    let event_loop = EventLoop::<()>::new();
    let window = WindowBuilder::new()
        .with_title("tairitsu-debug")
        .with_inner_size(tao::dpi::LogicalSize::new(1920.0, 1080.0))
        .with_visible(false)
        .with_decorations(false)
        .build(&event_loop)
        .map_err(|e| format!("Failed to create hidden window: {}", e))?;

    let webview = WebViewBuilder::new()
        .with_url("about:blank")
        .with_visible(false)
        .build(&window)
        .map_err(|e| format!("Failed to create WebView: {}", e))?;

    #[allow(clippy::arc_with_non_send_sync)]
    let webview: Arc<Mutex<Option<wry::WebView>>> = Arc::new(Mutex::new(Some(webview)));
    let cmd_rx: Arc<Mutex<mpsc::Receiver<ComCommand>>> = Arc::new(Mutex::new(rx));

    let wv = Arc::clone(&webview);
    let crx = Arc::clone(&cmd_rx);

    event_loop.run(
        move |event: Event<'_, ()>,
              _target: &tao::event_loop::EventLoopWindowTarget<()>,
              control_flow: &mut ControlFlow| {
            *control_flow = ControlFlow::Poll;
            match &event {
                Event::LoopDestroyed => return,
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }
                _ => {}
            }
            if let Ok(r) = crx.lock() {
                while let Ok(cmd) = r.try_recv() {
                    handle_command(cmd, &wv);
                }
            }
        },
    );
}

fn handle_command(cmd: ComCommand, webview: &Arc<Mutex<Option<wry::WebView>>>) {
    match cmd {
        ComCommand::Shutdown => {}

        ComCommand::Navigate { url } => {
            if let Ok(g) = webview.lock()
                && let Some(ref w) = *g
            {
                let _ = w.load_url(&url);
            }
        }

        ComCommand::EvaluateScript {
            script,
            response_tx,
        } => {
            let exec_js = format!(
                "(function(){{try{{window.__tairitsu_debug=window.__tairitsu_debug||{{}};window.__tairitsu_debug.__last_eval=JSON.stringify({})}}catch(e){{window.__tairitsu_debug=window.__tairitsu_debug||{{}};window.__tairitsu_debug.__last_eval=JSON.stringify({{error:String(e.message)}})}}}})()",
                script
            );
            if let Ok(g) = webview.lock() {
                if let Some(ref w) = *g {
                    let _ = w.evaluate_script(&exec_js);
                    if let Some(tx) = response_tx {
                        std::thread::sleep(std::time::Duration::from_millis(400));
                        let rb = "JSON.stringify(window.__tairitsu_debug&&window.__tairitsu_debug.__last_eval?window.__tairitsu_debug.__last_eval:'null')";
                        let (t2, r2) = mpsc::channel::<String>();
                        let tc = t2.clone();
                        let _ = w.evaluate_script_with_callback(rb, move |v| {
                            let _ = tc.send(v);
                        });
                        match r2.recv_timeout(std::time::Duration::from_secs(5)) {
                            Ok(v) => {
                                let _ = tx.send(if v.is_empty() {
                                    r#"{"error":"timeout"}"#.to_string()
                                } else {
                                    v
                                });
                            }
                            Err(_) => {
                                let _ = tx.send(r#"{"error":"timeout"}"#.to_string());
                            }
                        }
                    }
                } else if let Some(tx) = response_tx {
                    let _ = tx.send(r#"{"error":"no_webview"}"#.into());
                }
            } else if let Some(tx) = response_tx {
                let _ = tx.send(r#"{"error":"locked"}"#.into());
            }
        }

        ComCommand::Click { selector } => {
            if let Ok(g) = webview.lock()
                && let Some(ref w) = *g
            {
                let sel = selector.replace('\'', "\\'").replace('"', "\\\"");
                let js = format!(
                    "(function(){{var el=document.querySelector('{}');if(el){{el.click();return{{ok:true}}}}return{{ok:false}}}})()",
                    sel
                );
                let _ = w.evaluate_script(&js);
            }
        }

        ComCommand::Screenshot { response_tx } => {
            if let Ok(g) = webview.lock() {
                if let Some(ref w) = *g {
                    let snapshot_js = include_str!("snapshot_js.inl");
                    #[allow(clippy::type_complexity)]
                    let (t2, r2): (
                        mpsc::Sender<Result<String, String>>,
                        mpsc::Receiver<Result<String, String>>,
                    ) = mpsc::channel();
                    let t2o = t2.clone();
                    let _ = w.evaluate_script_with_callback(snapshot_js, move |val| {
                        let _ = t2o.send(Ok(val));
                    });
                    let result: Result<String, String> =
                        match r2.recv_timeout(std::time::Duration::from_secs(10)) {
                            Ok(Ok(v)) => Ok(v),
                            Ok(Err(e)) => Err(e),
                            Err(_) => Err("timeout".to_string()),
                        };
                    if let Some(tx) = response_tx {
                        let _ = tx.send(result);
                    }
                } else if let Some(tx) = response_tx {
                    let _ = tx.send(Err("no webview".into()));
                }
            } else if let Some(tx) = response_tx {
                let _ = tx.send(Err("locked".into()));
            }
        }
    }
}

// ─────────────────────────────────────────────────────
// Request / Response types
// ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct NavigateResponse {
    pub ok: bool,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickRequest {
    pub selector: String,
}

#[derive(Debug, Serialize)]
pub struct ClickResponse {
    pub ok: bool,
    pub selector: String,
}

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest {
    pub script: String,
}

#[derive(Debug, Serialize)]
pub struct EvaluateResponse {
    pub result: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub snapshot: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotResponse {
    pub ok: bool,
    pub data: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConsoleResponse {
    pub entries: Vec<ConsoleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub level: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub port: u16,
    pub current_url: Option<String>,
    pub console_log_count: usize,
}

// ─────────────────────────────────────────────────────
// Axum handlers
// ─────────────────────────────────────────────────────

pub async fn handle_status(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::Json<StatusResponse> {
    axum::Json(StatusResponse {
        connected: true,
        port: state.port().await,
        current_url: None,
        console_log_count: 0,
    })
}

pub async fn handle_navigate(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<NavigateRequest>,
) -> axum::response::Result<axum::Json<NavigateResponse>, (axum::http::StatusCode, String)> {
    state.send_cmd(ComCommand::Navigate {
        url: body.url.clone(),
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    Ok(axum::Json(NavigateResponse {
        ok: true,
        url: body.url,
    }))
}

pub async fn handle_snapshot(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::response::Result<axum::Json<SnapshotResponse>, (axum::http::StatusCode, String)> {
    let snapshot_js = include_str!("snapshot_js.inl");
    let title_js = "document.title || ''";
    let snapshot = state.send_eval(snapshot_js).unwrap_or_default();
    let title = state
        .send_eval(title_js)
        .unwrap_or_else(|| "(unknown)".to_string());
    Ok(axum::Json(SnapshotResponse {
        snapshot: clean_json_string(snapshot),
        url: "(headless)".to_string(),
        title: clean_json_string(title),
    }))
}

pub async fn handle_click(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<ClickRequest>,
) -> axum::response::Result<axum::Json<ClickResponse>, (axum::http::StatusCode, String)> {
    state.send_cmd(ComCommand::Click {
        selector: body.selector.clone(),
    });
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    Ok(axum::Json(ClickResponse {
        ok: true,
        selector: body.selector,
    }))
}

pub async fn handle_evaluate(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<EvaluateRequest>,
) -> axum::response::Result<axum::Json<EvaluateResponse>, (axum::http::StatusCode, String)> {
    let raw = state
        .send_eval(&body.script)
        .unwrap_or_else(|| r#"{"error":"eval_failed"}"#.to_string());
    let cleaned = clean_json_string(raw);
    let value: serde_json::Value =
        serde_json::from_str(&cleaned).unwrap_or(serde_json::Value::String(cleaned));
    Ok(axum::Json(EvaluateResponse { result: value }))
}

pub async fn handle_screenshot(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::response::Result<axum::Json<ScreenshotResponse>, (axum::http::StatusCode, String)> {
    let (tx, rx) = mpsc::channel();
    state.send_cmd(ComCommand::Screenshot {
        response_tx: Some(tx),
    });
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(Ok(html)) => Ok(axum::Json(ScreenshotResponse {
            ok: true,
            data: Some(format!(
                "data:text/html;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(html.as_bytes())
            )),
            error: None,
        })),
        Ok(Err(e)) => Ok(axum::Json(ScreenshotResponse {
            ok: false,
            data: None,
            error: Some(e),
        })),
        Err(_) => Ok(axum::Json(ScreenshotResponse {
            ok: false,
            data: None,
            error: Some("timeout".into()),
        })),
    }
}

pub async fn handle_console(
    _state: axum::extract::State<DebugApiState>,
) -> axum::Json<ConsoleResponse> {
    axum::Json(ConsoleResponse { entries: vec![] })
}

/// Strip JSON quotes from a string that was double-encoded by WebView2 eval.
fn clean_json_string(raw: String) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('"')
        && trimmed.ends_with('"')
        && trimmed.len() >= 2
        && let Ok(unescaped) = serde_json::from_str::<String>(trimmed)
    {
        return unescaped;
    }
    raw
}
