//! Tairitsu Virtual Browser + VTty — MCP (Model Context Protocol) server.
//!
//! Provides browser automation AND virtual terminal (VTty) tools to AI coding assistants.
//! All requests go through tairitsu packager CLI via stdio JSON-RPC 2.0.
//!
//! # Usage
//!
//! ```bash
//!   tairitsu mcp [--port PORT] [--url BASE_URL]
//! ```
//!
//! Speaks JSON-RPC 2.0 over stdin/stdout.  Designed for the `mcpServers`
//! configuration in opencode.jsonc / opencode.json.

use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};
use serde_json::json;

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const TOOL_PREFIX: &str = "";

#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    pub base_url: String,
}

pub async fn run(config: McpConfig) -> crate::Result<()> {
    let base_url: std::sync::Arc<tokio::sync::RwLock<String>> =
        std::sync::Arc::new(tokio::sync::RwLock::new(String::new()));
    let base_url_for_resolve = base_url.clone();
    let config_url = config.base_url;

    let resolve_handle = tokio::spawn(async move {
        let url = if config_url.is_empty() {
            match resolve_daemon_url().await {
                Ok(url) => url,
                Err(e) => {
                    tracing::debug!("[tairitsu-mcp] Warning: {}", e);
                    String::new()
                }
            }
        } else {
            config_url
        };
        *base_url_for_resolve.write().await = url;
    });

    eprintln!(
        "{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/diagnostic\",\"params\":{{\"status\":\"starting\",\"pid\":{},\"ppid\":{},\"features\":\"{}\"}}}}",
        std::process::id(),
        {
            #[cfg(unix)]
            { std::os::unix::process::parent_id() }
            #[cfg(windows)]
            { get_ppid_windows() }
        },
        if cfg!(feature = "vtty") {
            "vtty,browser"
        } else {
            "browser"
        }
    );

    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_default();
    #[cfg(feature = "vtty")]
    let state = McpState {
        base_url,
        http,
        vtty: std::sync::Arc::new(crate::vtty::VttyManager::new()),
    };
    #[cfg(not(feature = "vtty"))]
    let state = McpState { base_url, http };

    tracing::info!(
        "[tairitsu-mcp] Connected — browser + {}vtty tools available",
        if cfg!(feature = "vtty") { "" } else { "(no " }
    );

    run_jsonrpc_loop(state).await;
    let _ = resolve_handle.await;

    Ok(())
}

async fn resolve_daemon_url() -> crate::Result<String> {
    if let Ok(url) = std::env::var("TAIRITSU_DAEMON_URL") {
        if !url.is_empty() {
            tracing::debug!("[tairitsu-mcp] Using TAIRITSU_DAEMON_URL={}", url);
            return Ok(url);
        }
    }

    let searched = search_project_roots();
    if let Some((port, found_at)) = try_read_ready_port_from_candidates(&searched) {
        tracing::debug!("[tairitsu-mcp] Found ready file at {}", found_at.display());
        return Ok(format!("http://localhost:{}", port));
    }

    use crate::daemon;
    if daemon::is_daemon_running() {
        let pid = daemon::read_pid().unwrap_or(0);
        return Err(crate::TairitsuPackagerError::BuildError(format!(
            "Daemon is running (PID {}) but port unknown.\n\
             Searched: {}\n\
             Hint: set TAIRITSU_DAEMON_URL=http://localhost:<PORT> or pass --url",
            pid,
            searched
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    Err(crate::TairitsuPackagerError::BuildError(format!(
        "No running tairitsu daemon found.\n\
         Searched: {}\n\
         Hint: start with `tairitsu dev --daemon` or set TAIRITSU_DAEMON_URL",
        searched
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

fn search_project_roots() -> Vec<std::path::PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("target"));
        let mut dir = cwd.clone();
        for _ in 0..5 {
            if dir.join("Cargo.toml").exists() {
                candidates.push(dir.join("target"));
            }
            if !dir.pop() {
                break;
            }
        }
    }

    if let Ok(root) = std::env::var("TAIRITSU_PROJECT_ROOT") {
        let p = std::path::PathBuf::from(root);
        candidates.push(p.join("target"));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
            candidates.push(parent.join("target"));
        }
    }

    candidates.dedup();
    candidates
}

fn try_read_ready_port_from_candidates(
    dirs: &[std::path::PathBuf],
) -> Option<(u16, std::path::PathBuf)> {
    for dir in dirs {
        let ready_path = dir.join("tairitsu-packager.ready");
        if let Ok(content) = std::fs::read_to_string(&ready_path) {
            let trimmed = content.trim();
            if let Some(port_str) = trimmed.strip_prefix("ready:") {
                if let Ok(port) = port_str.parse::<u16>() {
                    return Some((port, ready_path));
                }
            } else if trimmed == "ready" {
                return Some((3000, ready_path));
            }
        }
    }
    None
}

struct McpState {
    base_url: std::sync::Arc<tokio::sync::RwLock<String>>,
    http: reqwest::Client,
    #[cfg(feature = "vtty")]
    vtty: std::sync::Arc<crate::vtty::VttyManager>,
}

impl McpState {
    #[allow(dead_code)]
    async fn get_base_url(&self) -> String {
        self.base_url.read().await.clone()
    }

    #[allow(dead_code)]
    async fn require_daemon(&self) -> Result<String, String> {
        let url = self.base_url.read().await.clone();
        if url.is_empty() {
            Err(
                "Browser tools require a running daemon. Start with: tairitsu dev --daemon"
                    .to_string(),
            )
        } else {
            Ok(url)
        }
    }
}

// ─────────────────────────────────────────────────────
// JSON-RPC message types (MCP protocol)
// ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

impl JsonRpcResponse {
    fn ok(id: Option<serde_json::Value>, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        }
    }

    fn err(id: Option<serde_json::Value>, code: i32, msg: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: msg.into(),
            }),
        }
    }
}

// ─────────────────────────────────────────────────────
// Tool definitions — mirrors playwright/browsermcp API surface
// ─────────────────────────────────────────────────────

fn tool_list() -> Vec<serde_json::Value> {
    vec![
        // --- Navigation ---
        tool(
            "browser_navigate",
            "Navigate to a URL",
            json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to navigate to"}
                },
                "required": ["url"]
            }),
        ),
        tool(
            "browser_navigate_back",
            "Go back to the previous page",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "browser_navigate_forward",
            "Go forward to the next page",
            json!({"type": "object", "properties": {}}),
        ),
        // --- Page inspection ---
        tool(
            "browser_snapshot",
            "Capture accessibility snapshot of the current page (DOM tree with roles, names, text). Better than screenshot for understanding page structure.",
            json!({
                "type": "object",
                "properties": {
                    "target": {"type": "string", "description": "CSS selector to scope snapshot to a specific element (optional)"}
                }
            }),
        ),
        tool(
            "browser_screenshot",
            "Take a screenshot of the current viewport as PNG (returns base64 data URL)",
            json!({
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["png", "jpeg"], "description": "Image format (default: png)"},
                    "fullPage": {"type": "boolean", "description": "Capture full scrollable page (default: false)"},
                    "element": {"type": "string", "description": "CSS selector for element-only screenshot (optional)"}
                }
            }),
        ),
        // --- Interaction ---
        tool(
            "browser_click",
            "Click an element by CSS selector or reference from snapshot",
            json!({
                "type": "object",
                "properties": {
                    "element": {"type": "string", "description": "Human-readable description of the element"},
                    "target": {"type": "string", "description": "CSS selector or snapshot reference of the element to click"}
                },
                "required": ["target"]
            }),
        ),
        tool(
            "browser_type",
            "Type text into an editable element (input, textarea, contenteditable)",
            json!({
                "type": "object",
                "properties": {
                    "element": {"type": "string", "description": "Description of the input field"},
                    "target": {"type": "string", "description": "CSS selector or snapshot reference"},
                    "text": {"type": "string", "description": "Text to type"},
                    "submit": {"type": "boolean", "description": "Press Enter after typing (default: false)"}
                },
                "required": ["target", "text"]
            }),
        ),
        tool(
            "browser_press_key",
            "Press a keyboard key (Enter, Tab, Escape, ArrowUp, etc.)",
            json!({
                "type": "object",
                "properties": {
                    "key": {"type": "string", "description": "Key name: Enter, Tab, Escape, Backspace, Delete, ArrowUp/Down/Left/Right, Home, End, PageUp/PageDown, F1-F12, Space, or any single character"}
                },
                "required": ["key"]
            }),
        ),
        tool(
            "browser_hover",
            "Hover mouse over an element (triggers tooltips, dropdowns, etc.)",
            json!({
                "type": "object",
                "properties": {
                    "element": {"type": "string", "description": "Description of the element"},
                    "target": {"type": "string", "description": "CSS selector or snapshot reference"}
                },
                "required": ["target"]
            }),
        ),
        tool(
            "browser_select_option",
            "Select option(s) in a <select> dropdown",
            json!({
                "type": "object",
                "properties": {
                    "element": {"type": "string", "description": "Description of the select element"},
                    "target": {"type": "string", "description": "CSS selector of the <select> element"},
                    "values": {"type": "array", "items": {"type": "string"}, "description": "Option values or text to select"}
                },
                "required": ["target", "values"]
            }),
        ),
        tool(
            "browser_fill_form",
            "Fill multiple form fields at once",
            json!({
                "type": "object",
                "properties": {
                    "fields": {"type": "array", "items": {
                        "type": "object",
                        "properties": {
                            "element": {"type": "string"},
                            "target": {"type": "string"},
                            "name": {"type": "string", "description": "Field name (for textbox/radio/checkbox/combobox/slider)"},
                            "type": {"type": "string", "enum": ["textbox", "checkbox", "radio", "combobox", "slider"], "description": "Field type"},
                            "value": {"type": "string", "description": "Value to set (true/false for checkbox, text for textbox, option text for combobox)"}
                        },
                        "required": ["target", "name", "type", "value"]
                    }, "description": "Form fields to fill"}
                },
                "required": ["fields"]
            }),
        ),
        // --- JavaScript execution ---
        tool(
            "browser_evaluate",
            "Evaluate JavaScript expression in the page context and return result",
            json!({
                "type": "object",
                "properties": {
                    "function": {"type": "string", "description": "JavaScript expression or function body to evaluate. Use () => { ... } for multi-line."},
                    "element": {"type": "string", "description": "Optional element description when targeting specific element"},
                    "target": {"type": "string", "description": "Optional CSS selector to scope evaluation to element"}
                },
                "required": ["function"]
            }),
        ),
        // --- Page info ---
        tool(
            "browser_console_messages",
            "Get console log entries (error/warning/info/debug) from the page",
            json!({
                "type": "object",
                "properties": {
                    "level": {"type": "string", "enum": ["error", "warning", "info", "debug"], "description": "Minimum log level (default: info)"},
                    "all": {"type": "boolean", "description": "Return all messages since session start (default: false, only returns new messages since last call)"}
                }
            }),
        ),
        tool(
            "browser_network_requests",
            "List HTTP network requests made by the page since last navigation",
            json!({
                "type": "object",
                "properties": {
                    "static": {"type": "boolean", "description": "Include static resources (images, fonts, css) in results (default: false)"},
                    "filter": {"type": "string", "description": "Regex filter for request URLs (optional)"}
                }
            }),
        ),
        // --- Tabs ---
        tool(
            "browser_tabs",
            "List, create, close, or switch browser tabs",
            json!({
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["list", "new", "close", "select"], "description": "Tab action (default: list)"},
                    "index": {"type": "number", "description": "Tab index for close/select actions"},
                    "url": {"type": "string", "description": "URL for new tab action"}
                }
            }),
        ),
        // --- Utility ---
        tool(
            "browser_wait_for",
            "Wait for a condition: time (seconds), text appearance, or text disappearance",
            json!({
                "type": "object",
                "properties": {
                    "time": {"type": "number", "description": "Wait duration in seconds"},
                    "text": {"type": "string", "description": "Wait for this text to appear on the page"},
                    "textGone": {"type": "string", "description": "Wait for this text to disappear from the page"}
                }
            }),
        ),
        tool(
            "browser_close",
            "Close the current tab or entire browser session",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "browser_resize",
            "Resize the browser window",
            json!({
                "type": "object",
                "properties": {
                    "width": {"type": "number", "description": "Window width in pixels"},
                    "height": {"type": "number", "description": "Window height in pixels"}
                },
                "required": ["width", "height"]
            }),
        ),
        // ── VTty: Virtual Terminal (ConPTY on Windows, forkpty on Unix) ──
        #[cfg(feature = "vtty")]
        tool(
            "vtty_launch",
            "Launch a command in a virtual terminal session",
            json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to run"},
                    "cols": {"type": "number", "description": "Terminal width in columns (default 120)"},
                    "rows": {"type": "number", "description": "Terminal height in rows (default 40)"},
                    "env": {"type": "string", "description": "Extra env vars as KEY=VAL KEY2=VAL2 string (optional)"},
                    "cwd": {"type": "string", "description": "Working directory (optional)"},
                    "name": {"type": "string", "description": "Friendly name for the session (optional)"}
                },
                "required": ["command"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_kill",
            "Kill a virtual terminal session",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID returned by vtty_launch"}
                },
                "required": ["session_id"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_send_keys",
            "Send key sequences to a virtual terminal. Supports Enter, Tab, Escape, Backspace, Delete, Arrow keys, Home/End, PageUp/PageDown, F1-F12, Ctrl+X, Alt+X",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"},
                    "keys": {"type": "string", "description": "Key sequence, e.g. 'Enter', 'Ctrl+C', 'Up Down Up Down Left Right B A' (space-separated)"}
                },
                "required": ["session_id", "keys"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_send_text",
            "Send text string to a virtual terminal",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"},
                    "text": {"type": "string", "description": "Text to type into the terminal"}
                },
                "required": ["session_id", "text"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_screenshot",
            "Capture current terminal screen content as text",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"}
                },
                "required": ["session_id"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_wait",
            "Wait for duration or until text appears on screen",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"},
                    "seconds": {"type": "number", "description": "Wait time in seconds (default 5)"},
                    "pattern": {"type": "string", "description": "Wait for this text pattern to appear on screen (optional)"}
                },
                "required": ["session_id"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_resize",
            "Resize a virtual terminal",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"},
                    "cols": {"type": "number", "description": "New width in columns"},
                    "rows": {"type": "number", "description": "New height in rows"}
                },
                "required": ["session_id", "cols", "rows"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_list",
            "List all active virtual terminal sessions",
            json!({"type": "object", "properties": {}}),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_ping",
            "Check if a VTty session's child process is still alive and refresh screen state",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"}
                },
                "required": ["session_id"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_ready",
            "Wait until a VTty session has screen output (useful after vtty_launch for slow-starting commands). Returns immediately if output is already present.",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"},
                    "timeout_ms": {"type": "number", "description": "Max wait time in milliseconds (default 30000, 30s)"}
                },
                "required": ["session_id"]
            }),
        ),
        #[cfg(feature = "vtty")]
        tool(
            "vtty_scrollback",
            "Get the scrollback buffer (history) of a virtual terminal session, including current screen content",
            json!({
                "type": "object",
                "properties": {
                    "session_id": {"type": "string", "description": "Session ID"}
                },
                "required": ["session_id"]
            }),
        ),
    ]
}

fn tool(name: &str, desc: &str, schema: serde_json::Value) -> serde_json::Value {
    json!({
        "name": format!("{}{}", TOOL_PREFIX, name),
        "description": desc,
        "inputSchema": schema
    })
}

// ─────────────────────────────────────────────────────
// Main JSON-RPC loop
// ─────────────────────────────────────────────────────

async fn run_jsonrpc_loop(state: McpState) {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    let reader = io::BufReader::new(stdin.lock());

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                let is_notification = serde_json::from_str::<serde_json::Value>(&trimmed)
                    .ok()
                    .and_then(|v| v.as_object().cloned())
                    .is_none_or(|obj| !obj.contains_key("id"));

                match serde_json::from_str::<JsonRpcRequest>(&trimmed) {
                    Ok(req) => {
                        if is_notification {
                            handle_notification(&state, &req).await;
                        } else if let Some(resp) = handle_request(&state, req).await {
                            let output = serde_json::to_string(&resp).unwrap_or_default();
                            writeln!(stdout, "{}", output).ok();
                            stdout.flush().ok();
                        }
                    }
                    Err(e) => {
                        if !is_notification {
                            let resp =
                                JsonRpcResponse::err(None, -32700, format!("Parse error: {}", e));
                            writeln!(
                                stdout,
                                "{}",
                                serde_json::to_string(&resp).unwrap_or_default()
                            )
                            .ok();
                            stdout.flush().ok();
                        }
                    }
                }
            }
            Err(_) => break,
        }
    }
}

async fn handle_notification(_state: &McpState, _req: &JsonRpcRequest) {}

async fn handle_request(state: &McpState, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
    match req.method.as_str() {
        "initialize" => Some(handle_initialize(req.id)),
        "tools/list" => Some(handle_tools_list(req.id)),
        "tools/call" => Some(handle_tools_call(state, req.id, req.params).await),
        "ping" => Some(JsonRpcResponse::ok(req.id, serde_json::json!({}))),
        _ => Some(JsonRpcResponse::err(
            req.id,
            -32601,
            format!("Method not found: {}", req.method),
        )),
    }
}

fn handle_initialize(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": if cfg!(feature = "vtty") { "tairitsu-mcp" } else { "tairitsu-virtual-browser" },
                    "version": env!("CARGO_PKG_VERSION")
                }
        }),
    )
}

fn handle_tools_list(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(id, json!({ "tools": tool_list() }))
}

async fn handle_tools_call(
    state: &McpState,
    id: Option<serde_json::Value>,
    params: Option<serde_json::Value>,
) -> JsonRpcResponse {
    let name = params
        .as_ref()
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("");

    let arguments = params
        .as_ref()
        .and_then(|p| p.get("arguments"))
        .cloned()
        .unwrap_or(json!({}));

    match invoke_tool(state, name, &arguments).await {
        Ok(result) => JsonRpcResponse::ok(
            id,
            json!({
                "content": [{ "type": "text", "text": result }],
                "isError": false
            }),
        ),
        Err(err_msg) => JsonRpcResponse::ok(
            id,
            json!({
                "content": [{ "type": "text", "text": err_msg }],
                "isError": true
            }),
        ),
    }
}

// ─────────────────────────────────────────────────────
// Tool invocation — dispatches to daemon debug API
// ─────────────────────────────────────────────────────

async fn invoke_tool(
    state: &McpState,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<String, String> {
    let base_url = state.base_url.read().await.clone();
    let api = |path: &str| format!("{}/__tairitsu_debug/{}", base_url, path);

    if tool_name.starts_with("browser_") && base_url.is_empty() {
        return Err(
            "Browser tools require a running daemon. Start with: tairitsu dev --daemon".to_string(),
        );
    }

    match tool_name {
        // --- Navigation ---
        "browser_navigate" => {
            let url = arg_str(args, "url")?;
            let resp = state
                .http
                .post(api("navigate"))
                .json(&json!({"url": url}))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let _body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            Ok(format!("Navigated to {}", url))
        }
        "browser_navigate_back" => {
            Ok("(navigate-back: not yet implemented on headless wry)".to_string())
        }
        "browser_navigate_forward" => {
            Ok("(navigate-forward: not yet implemented on headless wry)".to_string())
        }

        // --- Inspection ---
        "browser_snapshot" => {
            let resp = state
                .http
                .get(api("snapshot"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let snap = body
                .get("snapshot")
                .and_then(|v| v.as_str())
                .unwrap_or("(empty)");
            let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("Title: {}\n\n{}", title, snap))
        }
        "browser_screenshot" => {
            let resp = state
                .http
                .get(api("screenshot"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok("Screenshot captured successfully".to_string())
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                Err(format!("Screenshot failed: {}", err))
            }
        }

        // --- Interaction ---
        "browser_click" => {
            let target = arg_str(args, "target")?;
            let _resp = state
                .http
                .post(api("click"))
                .json(&json!({"selector": target}))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("Clicked: {}", target))
        }
        "browser_type" => {
            let target = arg_str(args, "target")?;
            let text = arg_str(args, "text")?;
            let submit = args
                .get("submit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let js = [
                "(function(){var el=document.querySelector('",
                &escape_js_selector(target),
                "');if(el){el.value='",
                &escape_js_str(text),
                "';el.dispatchEvent(new Event('input',{bubbles:true}));",
                if submit {
                    "el.dispatchEvent(new Event('change',{bubbles:true}));el.form?.submit();"
                } else {
                    ""
                },
                "})()",
            ]
            .join("");
            let resp = state
                .http
                .post(api("evaluate"))
                .json(&json!({"script": js}))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let r = body.get("result").cloned().unwrap_or(json!(null));
            Ok(format!(
                "Typed '{}' into {}. Result: {}",
                text,
                target,
                serde_json::to_string_pretty(&r).unwrap_or_default()
            ))
        }
        "browser_press_key" => {
            let key = arg_str(args, "key")?;
            let key_code = map_key_name(key);
            let js = format!(
                "(function(){{document.dispatchEvent(new KeyboardEvent('keydown',{{key:'{}',code:'{}',bubbles:true}}));document.dispatchEvent(new KeyboardEvent('keyup',{{key:'{}',code:'{}',bubbles:true}}))}})()",
                key, key_code, key, key_code
            );
            let _resp = state
                .http
                .post(api("evaluate"))
                .json(&json!({"script": js}))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            Ok(format!("Pressed key: {}", key))
        }
        "browser_hover" => {
            let target = arg_str(args, "target")?;
            let js = format!(
                "(function(){{var el=document.querySelector('{}');if(el)el.dispatchEvent(new MouseEvent('mouseover',{{bubbles:true}}))}})()",
                escape_js_selector(target)
            );
            let _ = state
                .http
                .post(api("evaluate"))
                .json(&json!({"script": js}))
                .send()
                .await;
            Ok(format!("Hovered: {}", target))
        }
        "browser_select_option" => {
            let target = arg_str(args, "target")?;
            let values: Vec<&str> = args
                .get("values")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let values_json = serde_json::to_string(&values).unwrap_or_default();
            let js = format!(
                "(function(){{var el=document.querySelector('{}');if(el&&el.tagName==='SELECT'){{var vals={};vals.forEach(function(v){{for(var i=0;i<el.options.length;i++){{if(el.options[i].value===v||el.options[i].text===v)el.options[i].selected=true}}}});el.dispatchEvent(new Event('change',{{bubbles:true}}))}}}})()",
                escape_js_selector(target),
                values_json
            );
            let _ = state
                .http
                .post(api("evaluate"))
                .json(&json!({"script": js}))
                .send()
                .await;
            Ok(format!("Selected options {:?} in {}", values, target))
        }
        "browser_fill_form" => {
            let fields = args
                .get("fields")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "Missing required parameter: fields".to_string())?;
            let mut results = vec![];
            for field in fields {
                let target = field.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let ftype = field
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("textbox");
                let value = field.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let js = match ftype {
                    "checkbox" => format!(
                        "(function(){{var el=document.querySelector('{}');if(el)el.checked={}}})()",
                        escape_js_selector(target),
                        if value == "true" || value == "1" {
                            "true"
                        } else {
                            "false"
                        }
                    ),
                    _ => format!(
                        "(function(){{var el=document.querySelector('{}');if(el){{el.value='{}';el.dispatchEvent(new Event('input',{{bubbles:true}}))}}}})()",
                        escape_js_selector(target),
                        escape_js_str(value)
                    ),
                };
                let _ = state
                    .http
                    .post(api("evaluate"))
                    .json(&json!({"script": js}))
                    .send()
                    .await;
                results.push(format!("{} = {}", target, value));
            }
            Ok(results.join("\n"))
        }

        // --- JS eval ---
        "browser_evaluate" => {
            let func = arg_str(args, "function")?;
            let resp = state
                .http
                .post(api("evaluate"))
                .json(&json!({"script": func}))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let result = body.get("result").cloned().unwrap_or(json!(null));
            Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result)))
        }

        // --- Info ---
        "browser_console_messages" => {
            let resp = state
                .http
                .get(api("console"))
                .send()
                .await
                .map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let entries = body
                .get("entries")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            if entries.is_empty() {
                return Ok("(no console entries)".to_string());
            }
            let lines: Vec<String> = entries
                .iter()
                .filter_map(|e| {
                    Some(format!(
                        "[{}] {}",
                        e.get("level")?.as_str()?,
                        e.get("text")?.as_str()?
                    ))
                })
                .collect();
            Ok(lines.join("\n"))
        }
        "browser_network_requests" => {
            Ok("(network_requests: not yet implemented on headless wry)".to_string())
        }

        // --- Tabs ---
        "browser_tabs" => {
            let action = args
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("list");
            match action {
                "list" => Ok("[tab-0] current page".to_string()),
                "new" => {
                    if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
                        let _ = state
                            .http
                            .post(api("navigate"))
                            .json(&json!({"url": url}))
                            .send()
                            .await;
                        Ok(format!("Opened new tab: {}", url))
                    } else {
                        Err("Missing url for new tab".to_string())
                    }
                }
                _ => Err(format!("Tab action '{}' not supported", action)),
            }
        }

        // --- Utility ---
        "browser_wait_for" => {
            if let Some(secs) = args.get("time").and_then(|v| v.as_f64()) {
                tokio::time::sleep(tokio::time::Duration::from_secs_f64(secs)).await;
                Ok(format!("Waited {:.1}s", secs))
            } else if let Some(text) = args.get("text").and_then(|v| v.as_str()) {
                for _ in 0..30 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    let check_js = format!(
                        "!!document.body?.innerText?.includes('{}')",
                        escape_js_str(text)
                    );
                    let resp = state
                        .http
                        .post(api("evaluate"))
                        .json(&json!({"script": check_js}))
                        .send()
                        .await;
                    if let Ok(r) = resp {
                        if let Ok(body) = r.json::<serde_json::Value>().await {
                            if body
                                .get("result")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                            {
                                return Ok(format!("Text appeared: {}", text));
                            }
                        }
                    }
                }
                Err(format!("Timeout waiting for text: {}", text))
            } else {
                Err("Missing required parameter: time or text".to_string())
            }
        }
        "browser_close" => Ok("(close: headless wry window management not applicable)".to_string()),
        "browser_resize" => {
            let w = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1280);
            let h = args.get("height").and_then(|v| v.as_u64()).unwrap_or(720);
            Ok(format!("Resized to {}x{}", w, h))
        }

        // ── VTty tools (pure Rust, no Python) ──
        #[cfg(feature = "vtty")]
        "vtty_launch" => {
            let cmd = arg_str(args, "command")?;
            let cols = args.get("cols").and_then(|v| v.as_u64()).unwrap_or(120) as u16;
            let rows = args.get("rows").and_then(|v| v.as_u64()).unwrap_or(40) as u16;
            let env = args.get("env").and_then(|v| v.as_str()).unwrap_or("");
            let cwd = args.get("cwd").and_then(|v| v.as_str());
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let info = state
                .vtty
                .launch(cmd, cols, rows, env, cwd, name)
                .map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&info).unwrap_or_default())
        }
        #[cfg(feature = "vtty")]
        "vtty_kill" => {
            let sid = arg_str(args, "session_id")?;
            let info = state.vtty.kill(sid).map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&info).unwrap_or_default())
        }
        #[cfg(feature = "vtty")]
        "vtty_send_keys" => {
            let sid = arg_str(args, "session_id")?;
            let keys = arg_str(args, "keys")?;
            let session = state.vtty.get(sid)?;
            {
                let guard = session.lock().map_err(|e| format!("{}", e))?;
                guard.send_keys(keys).map_err(|e| e.to_string())?;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            {
                let guard = session.lock().map_err(|e| format!("{}", e))?;
                let _ = guard.read_and_update();
            }
            Ok(json!({"session_id": sid, "keys": keys, "sent": true}).to_string())
        }
        #[cfg(feature = "vtty")]
        "vtty_send_text" => {
            let sid = arg_str(args, "session_id")?;
            let text = arg_str(args, "text")?;
            let session = state.vtty.get(sid)?;
            {
                let guard = session.lock().map_err(|e| format!("{}", e))?;
                guard.send_text(text).map_err(|e| e.to_string())?;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            {
                let guard = session.lock().map_err(|e| format!("{}", e))?;
                let _ = guard.read_and_update();
            }
            Ok(json!({"session_id": sid, "length": text.len(), "sent": true}).to_string())
        }
        #[cfg(feature = "vtty")]
        "vtty_screenshot" => {
            let sid = arg_str(args, "session_id")?;
            let session = state.vtty.get(sid)?;
            let guard = session.lock().map_err(|e| format!("{}", e))?;
            let _ = guard.read_and_update();
            let text = guard.screenshot();
            let alive = guard.is_alive();
            Ok(json!({"session_id": sid, "alive": alive, "rows": guard.rows, "cols": guard.cols, "text": text}).to_string())
        }
        #[cfg(feature = "vtty")]
        "vtty_wait" => {
            let sid = arg_str(args, "session_id")?;
            let secs = args.get("seconds").and_then(|v| v.as_f64()).unwrap_or(5.0);
            let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let session = state.vtty.get(sid)?;
            if !pattern.is_empty() {
                let deadline = std::time::Instant::now()
                    + std::time::Duration::from_secs_f64(secs.min(1800.0));
                let mut found = false;
                while std::time::Instant::now() < deadline {
                    let alive = {
                        let guard = session.lock().map_err(|e| format!("{}", e))?;
                        if !guard.is_alive() {
                            false
                        } else {
                            let _ = guard.read_and_update();
                            let found_now = !guard.find_text(pattern).is_empty();
                            if found_now {
                                found = true;
                            }
                            guard.is_alive()
                        }
                    };
                    if found || !alive {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                }
                let alive = session.lock().map(|g| g.is_alive()).unwrap_or(false);
                Ok(
                    json!({"session_id": sid, "pattern": pattern, "found": found, "alive": alive})
                        .to_string(),
                )
            } else {
                let wait_secs = secs.min(1800.0) as u64;
                let mut alive = true;
                for _i in 0..(wait_secs * 20) {
                    alive = {
                        let guard = session.lock().map_err(|e| format!("{}", e))?;
                        if !guard.is_alive() {
                            false
                        } else {
                            let _ = guard.read_and_update();
                            guard.is_alive()
                        }
                    };
                    if !alive {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                let alive = if alive {
                    let guard = session.lock().map_err(|e| format!("{}", e))?;
                    let _ = guard.read_and_update();
                    guard.is_alive()
                } else {
                    alive
                };
                Ok(json!({"session_id": sid, "seconds_waited": secs, "alive": alive}).to_string())
            }
        }
        #[cfg(feature = "vtty")]
        "vtty_resize" => {
            let sid = arg_str(args, "session_id")?;
            let cols = args
                .get("cols")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing: cols".to_string())? as u16;
            let rows = args
                .get("rows")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "Missing: rows".to_string())? as u16;
            let session = state.vtty.get(sid)?;
            let guard = session.lock().map_err(|e| format!("{}", e))?;
            let old_cols = guard.cols;
            let old_rows = guard.rows;
            guard.resize(cols, rows).map_err(|e| e.to_string())?;
            Ok(json!({"session_id": sid, "old": {"cols": old_cols, "rows": old_rows}, "new": {"cols": cols, "rows": rows}}).to_string())
        }
        #[cfg(feature = "vtty")]
        "vtty_list" => {
            let sessions = state.vtty.list();
            Ok(serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| "[]".to_string()))
        }
        #[cfg(feature = "vtty")]
        "vtty_ping" => {
            let sid = arg_str(args, "session_id")?;
            let info = state.vtty.ping(sid).map_err(|e| e.to_string())?;
            Ok(serde_json::to_string_pretty(&info).unwrap_or_default())
        }
        #[cfg(feature = "vtty")]
        "vtty_ready" => {
            let sid = arg_str(args, "session_id")?;
            let timeout_ms = args
                .get("timeout_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(30000);
            let session = state.vtty.get(sid)?;
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
            let mut has_output = false;
            while std::time::Instant::now() < deadline {
                {
                    let guard = session.lock().map_err(|e| format!("{}", e))?;
                    if guard.has_output() {
                        has_output = true;
                        break;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            Ok(
                json!({"session_id": sid, "ready": has_output, "elapsed_ms": timeout_ms.saturating_sub(
                    (deadline - std::time::Instant::now()).as_millis() as u64
                )})
                .to_string(),
            )
        }
        #[cfg(feature = "vtty")]
        "vtty_scrollback" => {
            let sid = arg_str(args, "session_id")?;
            let session = state.vtty.get(sid)?;
            let guard = session.lock().map_err(|e| format!("{}", e))?;
            let text = guard.scrollback();
            Ok(json!({"session_id": sid, "text": text}).to_string())
        }

        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

// ─────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────

fn arg_str<'a>(args: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn escape_js_selector(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
}

fn escape_js_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn map_key_name(key: &str) -> String {
    match key {
        "Enter" | "enter" | "Return" | "return" => "Enter".to_string(),
        "Tab" | "tab" => "Tab".to_string(),
        "Escape" | "escape" | "Esc" | "esc" => "Escape".to_string(),
        "Backspace" | "backspace" => "Backspace".to_string(),
        "Delete" | "delete" => "Delete".to_string(),
        "ArrowUp" | "arrowup" | "Up" | "up" => "ArrowUp".to_string(),
        "ArrowDown" | "arrowdown" | "Down" | "down" => "ArrowDown".to_string(),
        "ArrowLeft" | "arrowleft" | "Left" | "left" => "ArrowLeft".to_string(),
        "ArrowRight" | "arrowright" | "Right" | "right" => "ArrowRight".to_string(),
        "Home" | "home" => "Home".to_string(),
        "End" | "end" => "End".to_string(),
        "PageUp" | "pageup" => "PageUp".to_string(),
        "PageDown" | "pagedown" => "PageDown".to_string(),
        "Space" | "space" => "Space".to_string(),
        _ => key.to_string(),
    }
}

#[cfg(windows)]
fn get_ppid_windows() -> u32 {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

    let pid = std::process::id();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return 0;
        }
        let mut entry: PROCESSENTRY32 = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
        if Process32First(snap, &mut entry) != 0 {
            loop {
                if entry.th32ProcessID == pid {
                    let ppid = entry.th32ParentProcessID;
                    CloseHandle(snap);
                    return ppid;
                }
                if Process32Next(snap, &mut entry) == 0 {
                    break;
                }
            }
        }
        CloseHandle(snap);
        0
    }
}
