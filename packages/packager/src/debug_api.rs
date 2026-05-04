//! Debug API for browser automation — exposed by the dev server daemon.
//!
//! Provides HTTP endpoints (`/__tairitsu_debug/*`) that `tairitsu mcp` (or any
//! HTTP client) can call to inspect and interact with the running application
//! via Chrome DevTools Protocol (CDP).

use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
pub struct DebugApiState {
    inner: Arc<tokio::sync::Mutex<DebugApiInner>>,
}

impl DebugApiState {
    pub fn new(port: u16) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Mutex::new(DebugApiInner {
                browser: None,
                port,
                console_logs: Vec::new(),
                max_console_entries: 500,
            })),
        }
    }

    pub async fn set_port(&self, port: u16) {
        let mut inner = self.inner.lock().await;
        inner.port = port;
    }

    pub async fn launch_browser(&self, url: &str, headless: bool) -> crate::Result<()> {
        let mut inner = self.inner.lock().await;
        #[cfg(feature = "headless_chrome")]
        {
            use headless_chrome::LaunchOptionsBuilder;

            let launch_opts = if headless {
                LaunchOptionsBuilder::default()
                    .sandbox(true)
                    .headless(true)
                    .build()
            } else {
                LaunchOptionsBuilder::default()
                    .sandbox(true)
                    .headless(false)
                    .args(vec![std::ffi::OsStr::new("--auto-open-devtools-for-tabs")])
                    .build()
            }.map_err(|e| {
                crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to build Chrome launch options: {}",
                    e
                ))
            })?;

            let browser =
                headless_chrome::Browser::new(launch_opts).map_err(|e| {
                    crate::TairitsuPackagerError::BuildError(format!(
                        "Failed to launch Chrome: {}. Is Chrome/Chromium installed?",
                        e
                    ))
                })?;

            let tab = browser.new_tab().map_err(|e| {
                crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to open new tab: {}",
                    e
                ))
            })?;

            tab.navigate_to(url).map_err(|e| {
                crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to navigate to {}: {}",
                    url, e
                ))
            })?;

            inner.browser = Some(BrowserSession {
                _browser: browser,
                tab,
                current_url: url.to_string(),
            });

            crate::log_ok!("Debug browser launched: {} (headless={})", url, headless);
            Ok(())
        }
        #[cfg(not(feature = "headless_chrome"))]
        {
            let _ = (url, headless);
            Err(crate::TairitsuPackagerError::BuildError(
                "debug-api feature not enabled".to_string(),
            ))
        }
    }

    pub async fn is_browser_connected(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.browser.is_some()
    }

    pub async fn port(&self) -> u16 {
        let inner = self.inner.lock().await;
        inner.port
    }
}

struct DebugApiInner {
    browser: Option<BrowserSession>,
    port: u16,
    console_logs: Vec<ConsoleEntry>,
    max_console_entries: usize,
}

struct BrowserSession {
    #[cfg(feature = "headless_chrome")]
    _browser: headless_chrome::Browser,
    #[cfg(feature = "headless_chrome")]
    tab: std::sync::Arc<headless_chrome::browser::tab::Tab>,
    current_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub level: String,
    pub text: String,
    pub timestamp: String,
}

// ─────────────────────────────────────────────────────
// Request / Response types
// ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct NavigateRequest { pub url: String }

#[derive(Debug, Serialize)]
pub struct NavigateResponse { pub ok: bool, pub url: String }

#[derive(Debug, Deserialize)]
pub struct ClickRequest { pub selector: String }

#[derive(Debug, Serialize)]
pub struct ClickResponse { pub ok: bool, pub selector: String }

#[derive(Debug, Deserialize)]
pub struct EvaluateRequest { pub script: String }

#[derive(Debug, Serialize)]
pub struct EvaluateResponse { pub result: serde_json::Value }

#[derive(Debug, Serialize)]
pub struct SnapshotResponse { pub snapshot: String, pub url: String, pub title: String }

#[derive(Debug, Serialize)]
pub struct ScreenshotResponse { pub ok: bool, pub data: Option<String>, pub error: Option<String> }

#[derive(Debug, Serialize)]
pub struct ConsoleResponse { pub entries: Vec<ConsoleEntry> }

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub connected: bool,
    pub port: u16,
    pub current_url: Option<String>,
    pub console_log_count: usize,
}

// ─────────────────────────────────────────────────────
// Axum handlers (wrap sync CDP calls)

pub async fn handle_status(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::Json<StatusResponse> {
    let inner = state.inner.lock().await;
    axum::Json(StatusResponse {
        connected: inner.browser.is_some(),
        port: inner.port,
        current_url: inner.browser.as_ref().map(|b| b.current_url.clone()),
        console_log_count: inner.console_logs.len(),
    })
}

pub async fn handle_navigate(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<NavigateRequest>,
) -> axum::response::Result<axum::Json<NavigateResponse>, (axum::http::StatusCode, String)> {
    let mut inner = state.inner.lock().await;
    match inner.browser.as_mut() {
        Some(session) => {
            #[cfg(feature = "headless_chrome")]
            {
                if let Err(e) = session.tab.navigate_to(&body.url) {
                    return Err((axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Navigation failed: {}", e)));
                }
                session.current_url = body.url.clone();
                Ok(axum::Json(NavigateResponse { ok: true, url: body.url }))
            }
            #[cfg(not(feature = "headless_chrome"))]
            Err((axum::http::StatusCode::NOT_IMPLEMENTED, "debug-api feature not enabled".into()))
        }
        None => Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "No browser session".into())),
    }
}

pub async fn handle_snapshot(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::response::Result<axum::Json<SnapshotResponse>, (axum::http::StatusCode, String)> {
    let inner = state.inner.lock().await;
    match inner.browser.as_ref() {
        Some(session) => {
            #[cfg(feature = "headless_chrome")]
            {
                let js = r#"(function(){
                    function walk(el,depth){
                        var indent='  '.repeat(depth);
                        var tag=(el.tagName||'').toLowerCase();
                        if(['#text','#comment','script','style','link','meta','noscript'].includes(tag)) return '';
                        var role=el.getAttribute('role')||'';
                        var label=el.getAttribute('aria-label')||'';
                        var txt=(el.textContent||'').trim().slice(0,50);
                        var line=indent+tag;
                        if(role&&role!=='presentation') line+='['+role+']';
                        var name=label||txt;
                        if(name) line+=' "'+name.replace(/"/g,"'")+'"';
                        var children='';
                        for(var i=0;i<el.children.length;i++) children+=walk(el.children[i],depth+1);
                        return children?line+'\n'+children:line;
                    }
                    return walk(document.documentElement,0)||'(empty)';
                })()"#;

                let eval_result = session.tab.evaluate(js, false).map_err(|e| {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Snapshot failed: {}", e))
                })?;

                let snapshot = eval_result.value
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "(empty)".into());

                let title = session.tab.get_title()
                    .unwrap_or_default();

                Ok(axum::Json(SnapshotResponse {
                    snapshot,
                    url: session.current_url.clone(),
                    title,
                }))
            }
            #[cfg(not(feature = "headless_chrome"))]
            Err((axum::http::StatusCode::NOT_IMPLEMENTED, "debug-api feature not enabled".into()))
        }
        None => Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "No browser session".into())),
    }
}

pub async fn handle_click(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<ClickRequest>,
) -> axum::response::Result<axum::Json<ClickResponse>, (axum::http::StatusCode, String)> {
    let inner = state.inner.lock().await;
    match inner.browser.as_ref() {
        Some(session) => {
            #[cfg(feature = "headless_chrome")]
            {
                let sel = body.selector.replace('\'', "\\'");
                let js = format!(
                    r#"(function(){{var el=document.querySelector('{sel}');if(!el)return{{ok:false,error:'not found'}};el.click();return{{ok:true}}}})()"#
                );
                let result = session.tab.evaluate(&js, false).map_err(|e| {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Evaluate failed: {}", e))
                })?;
                let ok = result.value
                    .map(|v| v.get("ok").and_then(|v| v.as_bool()).unwrap_or(false))
                    .unwrap_or(false);
                Ok(axum::Json(ClickResponse { ok, selector: body.selector }))
            }
            #[cfg(not(feature = "headless_chrome"))]
            Err((axum::http::StatusCode::NOT_IMPLEMENTED, "debug-api feature not enabled".into()))
        }
        None => Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "No browser session".into())),
    }
}

pub async fn handle_evaluate(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
    axum::Json(body): axum::Json<EvaluateRequest>,
) -> axum::response::Result<axum::Json<EvaluateResponse>, (axum::http::StatusCode, String)> {
    let inner = state.inner.lock().await;
    match inner.browser.as_ref() {
        Some(session) => {
            #[cfg(feature = "headless_chrome")]
            {
                let eval_result = session.tab.evaluate(&body.script, false).map_err(|e| {
                    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Evaluate failed: {}", e))
                })?;
                Ok(axum::Json(EvaluateResponse {
                    result: eval_result.value.unwrap_or(serde_json::Value::Null),
                }))
            }
            #[cfg(not(feature = "headless_chrome"))]
            Err((axum::http::StatusCode::NOT_IMPLEMENTED, "debug-api feature not enabled".into()))
        }
        None => Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "No browser session".into())),
    }
}

pub async fn handle_screenshot(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::response::Result<axum::Json<ScreenshotResponse>, (axum::http::StatusCode, String)> {
    let inner = state.inner.lock().await;
    match inner.browser.as_ref() {
        Some(session) => {
            #[cfg(feature = "headless_chrome")]
            {
                use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
                match session.tab.capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true) {
                    Ok(png_data) => {
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
                        Ok(axum::Json(ScreenshotResponse { ok: true, data: Some(b64), error: None }))
                    }
                    Err(e) => Ok(axum::Json(ScreenshotResponse {
                        ok: false,
                        data: None,
                        error: Some(format!("{}", e)),
                    })),
                }
            }
            #[cfg(not(feature = "headless_chrome"))]
            Err((axum::http::StatusCode::NOT_IMPLEMENTED, "debug-api feature not enabled".into()))
        }
        None => Err((axum::http::StatusCode::SERVICE_UNAVAILABLE, "No browser session".into())),
    }
}

pub async fn handle_console(
    axum::extract::State(state): axum::extract::State<DebugApiState>,
) -> axum::Json<ConsoleResponse> {
    let inner = state.inner.lock().await;
    axum::Json(ConsoleResponse { entries: inner.console_logs.clone() })
}
