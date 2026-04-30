//! Debug/inspection API server for agent-driven browser automation.
//!
//! When `--debug` is passed to `tairitsu dev`, this module spawns an Axum
//! server on a separate port (default: dev-port + 1) that exposes endpoints
//! for screenshots, DOM queries, click/input simulation, and JS evaluation.
//!
//! Agents connect via HTTP and follow the protocol documented in
//! `docs/en/skills/debug-agent.md`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

use crate::config::Config;

const DEBUG_API_VERSION: &str = "0.1.0";

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NavigateRequest {
    url: String,
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
#[allow(dead_code)]
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
}

/// Shared state for the debug server
#[derive(Clone)]
struct DebugState {
    config: Config,
    dev_port: u16,
    debug_port: u16,
    start_time: Instant,
    base_url: String,
    console_log: Arc<RwLock<Vec<ConsoleEntry>>>,
}

impl DebugState {
    fn new(config: Config, dev_port: u16, debug_port: u16) -> Self {
        Self {
            base_url: format!("http://localhost:{}", dev_port),
            config,
            dev_port,
            debug_port,
            start_time: Instant::now(),
            console_log: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }
}

pub async fn start_debug_server(
    config: &Config,
    dev_port: u16,
    debug_port: u16,
) -> crate::Result<()> {
    let state = DebugState::new(config.clone(), dev_port, debug_port);
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

    crate::log_ok!(
        "Debug API server listening on http://localhost:{}",
        debug_port
    );
    crate::log_info!("Debug endpoints: /health, /info, /navigate, /screenshot, /click, /type, /evaluate, /console, /dom");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler(State(state): State<DebugState>) -> impl IntoResponse {
    ResponseJson(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        version: crate::VERSION.into(),
        api_version: DEBUG_API_VERSION.into(),
        uptime_secs: state.uptime_secs(),
    }))
}

async fn info_handler(State(state): State<DebugState>) -> impl IntoResponse {
    let info = InfoResponse {
        version: crate::VERSION.into(),
        api_version: DEBUG_API_VERSION.into(),
        dev_port: state.dev_port,
        debug_port: state.debug_port,
        dist_dir: state.config.build.output_dir.display().to_string(),
        package_name: state.config.package.name.clone(),
        pid: std::process::id(),
        started_at_iso: chrono::Utc::now().to_rfc3339(),
        uptime_secs: state.uptime_secs(),
        browser_connected: false,
    };
    ResponseJson(ApiResponse::ok(info))
}

async fn navigate_handler(
    State(state): State<DebugState>,
    Json(req): Json<NavigateRequest>,
) -> impl IntoResponse {
    let target_url = if req.url.starts_with("http") {
        req.url
    } else {
        format!("{}{}", state.base_url, req.url)
    };

    ResponseJson(ApiResponse::ok(NavigateResponse {
        url: target_url.clone(),
        title: String::new(),
    }))
}

async fn screenshot_handler(
    State(state): State<DebugState>,
    Json(params): Json<ScreenshotParams>,
) -> impl IntoResponse {
    let _ = (&state, &params);
    ResponseJson(ApiResponse::<ScreenshotResponse>::err(
        "Browser not connected. Start with --debug-browser or ensure headless chromium is available.",
    ))
}

async fn click_handler(
    State(_state): State<DebugState>,
    Json(req): Json<ClickRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<serde_json::Value>>) {
    let _ = req;
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::<serde_json::Value>::err(
            "Browser not connected.",
        )),
    )
}

async fn type_handler(
    State(_state): State<DebugState>,
    Json(req): Json<TypeRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<serde_json::Value>>) {
    let _ = req;
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::<serde_json::Value>::err(
            "Browser not connected.",
        )),
    )
}

async fn evaluate_handler(
    State(_state): State<DebugState>,
    Json(req): Json<EvaluateRequest>,
) -> (StatusCode, ResponseJson<ApiResponse<serde_json::Value>>) {
    let _ = req;
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::<serde_json::Value>::err(
            "Browser not connected.",
        )),
    )
}

async fn console_handler(
    State(state): State<DebugState>,
) -> impl IntoResponse {
    let log = state.console_log.read().await;
    ResponseJson(ApiResponse::ok(ConsoleResponse {
        entries: log.clone(),
    }))
}

async fn dom_query_handler(
    State(_state): State<DebugState>,
    Query(params): Query<DomQueryParams>,
) -> (StatusCode, ResponseJson<ApiResponse<DomNodeResponse>>) {
    let _ = params;
    (
        StatusCode::SERVICE_UNAVAILABLE,
        ResponseJson(ApiResponse::err("Browser not connected.")),
    )
}
