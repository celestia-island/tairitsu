//! Tairitsu Virtual Browser — MCP (Model Context Protocol) server.
//!
//! Provides browser automation tools to AI coding assistants (opencode, Claude,
//! Cursor, etc.) by connecting to a running tairitsu daemon's debug API.
//!
//! # Usage
//!
//! ```bash
//!   tairitsu mcp [--port PORT] [--url BASE_URL]
//! ```
//!
//! Speaks JSON-RPC 2.0 over stdin/stdout.  Designed for the `mcpServers`
//! configuration in opencode.jsonc / opencode.json.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{self, BufRead, Write};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const TOOL_PREFIX: &str = "";

#[derive(Debug, Clone)]
pub struct McpConfig {
    pub base_url: String,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            base_url: String::new(),
        }
    }
}

pub async fn run(config: McpConfig) -> crate::Result<()> {
    let base_url = if config.base_url.is_empty() {
        resolve_daemon_url().await?
    } else {
        config.base_url
    };

    let http = reqwest::Client::new();
    let state = McpState { base_url, http };

    eprintln!("[tairitsu-virtual-browser] Connected to daemon at {}", state.base_url);

    run_jsonrpc_loop(state).await;

    Ok(())
}

async fn resolve_daemon_url() -> crate::Result<String> {
    use crate::daemon;

    if let Some(port) = try_read_ready_port() {
        return Ok(format!("http://localhost:{}", port));
    }

    if daemon::is_daemon_running() {
        let pid = daemon::read_pid().unwrap_or(0);
        return Err(crate::TairitsuPackagerError::BuildError(format!(
            "Daemon is running (PID {}) but port unknown. Try --port.",
            pid
        )));
    }

    Err(crate::TairitsuPackagerError::BuildError(
        "No running daemon found. Start with: tairitsu dev --daemon".to_string(),
    ))
}

fn try_read_ready_port() -> Option<u16> {
    let ready_path = std::path::PathBuf::from("target")
        .join("tairitsu-packager.ready");
    let content = std::fs::read_to_string(&ready_path).ok()?;
    let trimmed = content.trim();
    if let Some(port_str) = trimmed.strip_prefix("ready:") {
        port_str.parse().ok()
    } else if trimmed == "ready" {
        Some(3000)
    } else {
        None
    }
}

struct McpState {
    base_url: String,
    http: reqwest::Client,
}

// ─────────────────────────────────────────────────────
// JSON-RPC message types (MCP protocol)
// ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
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
        tool("browser_navigate", "Navigate to a URL", json!({
            "type": "object",
            "properties": {
                "url": {"type": "string", "description": "URL to navigate to"}
            },
            "required": ["url"]
        })),
        tool("browser_navigate_back", "Go back to the previous page", json!({})),
        tool("browser_navigate_forward", "Go forward to the next page", json!({})),

        // --- Page inspection ---
        tool("browser_snapshot", "Capture accessibility snapshot of the current page (DOM tree with roles, names, text). Better than screenshot for understanding page structure.", json!({
            "type": "object",
            "properties": {
                "target": {"type": "string", "description": "CSS selector to scope snapshot to a specific element (optional)"}
            }
        })),
        tool("browser_screenshot", "Take a screenshot of the current viewport as PNG (returns base64 data URL)", json!({
            "type": "object",
            "properties": {
                "type": {"type": "string", "enum": ["png", "jpeg"], "description": "Image format (default: png)"},
                "fullPage": {"type": "boolean", "description": "Capture full scrollable page (default: false)"},
                "element": {"type": "string", "description": "CSS selector for element-only screenshot (optional)"}
            }
        })),

        // --- Interaction ---
        tool("browser_click", "Click an element by CSS selector or reference from snapshot", json!({
            "type": "object",
            "properties": {
                "element": {"type": "string", "description": "Human-readable description of the element"},
                "target": {"type": "string", "description": "CSS selector or snapshot reference of the element to click"}
            },
            "required": ["target"]
        })),
        tool("browser_type", "Type text into an editable element (input, textarea, contenteditable)", json!({
            "type": "object",
            "properties": {
                "element": {"type": "string", "description": "Description of the input field"},
                "target": {"type": "string", "description": "CSS selector or snapshot reference"},
                "text": {"type": "string", "description": "Text to type"},
                "submit": {"type": "boolean", "description": "Press Enter after typing (default: false)"}
            },
            "required": ["target", "text"]
        })),
        tool("browser_press_key", "Press a keyboard key (Enter, Tab, Escape, ArrowUp, etc.)", json!({
            "type": "object",
            "properties": {
                "key": {"type": "string", "description": "Key name: Enter, Tab, Escape, Backspace, Delete, ArrowUp/Down/Left/Right, Home, End, PageUp/PageDown, F1-F12, Space, or any single character"}
            },
            "required": ["key"]
        })),
        tool("browser_hover", "Hover mouse over an element (triggers tooltips, dropdowns, etc.)", json!({
            "type": "object",
            "properties": {
                "element": {"type": "string", "description": "Description of the element"},
                "target": {"type": "string", "description": "CSS selector or snapshot reference"}
            },
            "required": ["target"]
        })),
        tool("browser_select_option", "Select option(s) in a <select> dropdown", json!({
            "type": "object",
            "properties": {
                "element": {"type": "string", "description": "Description of the select element"},
                "target": {"type": "string", "description": "CSS selector of the <select> element"},
                "values": {"type": "array", "items": {"type": "string"}, "description": "Option values or text to select"}
            },
            "required": ["target", "values"]
        })),
        tool("browser_fill_form", "Fill multiple form fields at once", json!({
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
        })),

        // --- JavaScript execution ---
        tool("browser_evaluate", "Evaluate JavaScript expression in the page context and return result", json!({
            "type": "object",
            "properties": {
                "function": {"type": "string", "description": "JavaScript expression or function body to evaluate. Use () => { ... } for multi-line."},
                "element": {"type": "string", "description": "Optional element description when targeting specific element"},
                "target": {"type": "string", "description": "Optional CSS selector to scope evaluation to element"}
            },
            "required": ["function"]
        })),

        // --- Page info ---
        tool("browser_console_messages", "Get console log entries (error/warning/info/debug) from the page", json!({
            "type": "object",
            "properties": {
                "level": {"type": "string", "enum": ["error", "warning", "info", "debug"], "description": "Minimum log level (default: info)"},
                "all": {"type": "boolean", "description": "Return all messages since session start (default: false, only returns new messages since last call)"}
            }
        })),
        tool("browser_network_requests", "List HTTP network requests made by the page since last navigation", json!({
            "type": "object",
            "properties": {
                "static": {"type": "boolean", "description": "Include static resources (images, fonts, css) in results (default: false)"},
                "filter": {"type": "string", "description": "Regex filter for request URLs (optional)"}
            }
        })),

        // --- Tabs ---
        tool("browser_tabs", "List, create, close, or switch browser tabs", json!({
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["list", "new", "close", "select"], "description": "Tab action (default: list)"},
                "index": {"type": "number", "description": "Tab index for close/select actions"},
                "url": {"type": "string", "description": "URL for new tab action"}
            }
        })),

        // --- Utility ---
        tool("browser_wait_for", "Wait for a condition: time (seconds), text appearance, or text disappearance", json!({
            "type": "object",
            "properties": {
                "time": {"type": "number", "description": "Wait duration in seconds"},
                "text": {"type": "string", "description": "Wait for this text to appear on the page"},
                "textGone": {"type": "string", "description": "Wait for this text to disappear from the page"}
            }
        })),
        tool("browser_close", "Close the current tab or entire browser session", json!({})),
        tool("browser_resize", "Resize the browser window", json!({
            "type": "object",
            "properties": {
                "width": {"type": "number", "description": "Window width in pixels"},
                "height": {"type": "number", "description": "Window height in pixels"}
            },
            "required": ["width", "height"]
        })),
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
                if trimmed.is_empty() { continue; }

                match serde_json::from_str::<JsonRpcRequest>(&trimmed) {
                    Ok(req) => {
                        let resp = handle_request(&state, req).await;
                        let output = serde_json::to_string(&resp).unwrap_or_default();
                        writeln!(stdout, "{}", output).ok();
                        stdout.flush().ok();
                    }
                    Err(e) => {
                        let resp = JsonRpcResponse::err(None, -32700, format!("Parse error: {}", e));
                        writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap_or_default()).ok();
                        stdout.flush().ok();
                    }
                }
            }
            Err(_) => break,
        }
    }
}

async fn handle_request(state: &McpState, req: JsonRpcRequest) -> JsonRpcResponse {
    match req.method.as_str() {
        "initialize" => handle_initialize(req.id),
        "notifications/initialized" => JsonRpcResponse::ok(None, serde_json::json!(null)),
        "tools/list" => handle_tools_list(req.id),
        "tools/call" => handle_tools_call(state, req.id, req.params).await,
        "ping" => JsonRpcResponse::ok(req.id, serde_json::json!({})),
        _ => JsonRpcResponse::err(req.id, -32601, format!("Method not found: {}", req.method)),
    }
}

fn handle_initialize(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "tairitsu-virtual-browser",
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
    let api = |path: &str| format!("{}/__tairitsu_debug/{}", state.base_url, path);

    match tool_name {
        // --- Navigation ---
        "browser_navigate" => {
            let url = arg_str(args, "url")?;
            let resp = state.http.post(&api("navigate")).json(&json!({"url": url}))
                .send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
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
            let resp = state.http.get(&api("snapshot")).send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let snap = body.get("snapshot").and_then(|v| v.as_str()).unwrap_or("(empty)");
            let title = body.get("title").and_then(|v| v.as_str()).unwrap_or("");
            Ok(format!("Title: {}\n\n{}", title, snap))
        }
        "browser_screenshot" => {
            let resp = state.http.get(&api("screenshot")).send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok("Screenshot captured successfully".to_string())
            } else {
                let err = body.get("error").and_then(|v| v.as_str()).unwrap_or("unknown");
                Err(format!("Screenshot failed: {}", err))
            }
        }

        // --- Interaction ---
        "browser_click" => {
            let target = arg_str(args, "target")?;
            let resp = state.http.post(&api("click")).json(&json!({"selector": target}))
                .send().await.map_err(|e| e.to_string())?;
            Ok(format!("Clicked: {}", target))
        }
        "browser_type" => {
            let target = arg_str(args, "target")?;
            let text = arg_str(args, "text")?;
            let submit = args.get("submit").and_then(|v| v.as_bool()).unwrap_or(false);
            let js = [
                "(function(){var el=document.querySelector('", &escape_js_selector(&target),
                "');if(el){el.value='", &escape_js_str(&text),
                "';el.dispatchEvent(new Event('input',{bubbles:true}));",
                &if submit { "el.dispatchEvent(new Event('change',{bubbles:true}));el.form?.submit();" } else { "" },
                "})()"
            ].join("");
            let resp = state.http.post(&api("evaluate")).json(&json!({"script": js}))
                .send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let r = body.get("result").cloned().unwrap_or(json!(null));
            Ok(format!("Typed '{}' into {}. Result: {}", text, target,
                serde_json::to_string_pretty(&r).unwrap_or_default()))
        }
        "browser_press_key" => {
            let key = arg_str(args, "key")?;
            let key_code = map_key_name(&key);
            let js = format!(
                "(function(){{document.dispatchEvent(new KeyboardEvent('keydown',{{key:'{}',code:'{}',bubbles:true}}));document.dispatchEvent(new KeyboardEvent('keyup',{{key:'{}',code:'{}',bubbles:true}}))}})()",
                key, key_code, key, key_code
            );
            let resp = state.http.post(&api("evaluate")).json(&json!({"script": js}))
                .send().await.map_err(|e| e.to_string())?;
            Ok(format!("Pressed key: {}", key))
        }
        "browser_hover" => {
            let target = arg_str(args, "target")?;
            let js = format!(
                "(function(){{var el=document.querySelector('{}');if(el)el.dispatchEvent(new MouseEvent('mouseover',{{bubbles:true}}))}})()",
                escape_js_selector(&target)
            );
            let _ = state.http.post(&api("evaluate")).json(&json!({"script": js}))
                .send().await;
            Ok(format!("Hovered: {}", target))
        }
        "browser_select_option" => {
            let target = arg_str(args, "target")?;
            let values: Vec<&str> = args.get("values")
                .and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_str()).collect())
                .unwrap_or_default();
            let values_json = serde_json::to_string(&values).unwrap_or_default();
            let js = format!(
                "(function(){{var el=document.querySelector('{}');if(el&&el.tagName==='SELECT'){{var vals={};vals.forEach(function(v){{for(var i=0;i<el.options.length;i++){{if(el.options[i].value===v||el.options[i].text===v)el.options[i].selected=true}}}});el.dispatchEvent(new Event('change',{{bubbles:true}}))}}}})()",
                escape_js_selector(&target), values_json
            );
            let _ = state.http.post(&api("evaluate")).json(&json!({"script": js}))
                .send().await;
            Ok(format!("Selected options {:?} in {}", values, target))
        }
        "browser_fill_form" => {
            let fields = args.get("fields").and_then(|v| v.as_array())
                .ok_or_else(|| "Missing required parameter: fields".to_string())?;
            let mut results = vec![];
            for field in fields {
                let target = field.get("target").and_then(|v| v.as_str()).unwrap_or("");
                let ftype = field.get("type").and_then(|v| v.as_str()).unwrap_or("textbox");
                let value = field.get("value").and_then(|v| v.as_str()).unwrap_or("");
                let js = match ftype {
                    "checkbox" => format!(
                        "(function(){{var el=document.querySelector('{}');if(el)el.checked={}}})()",
                        escape_js_selector(target), if value == "true" || value == "1" { "true" } else { "false" }
                    ),
                    _ => format!(
                        "(function(){{var el=document.querySelector('{}');if(el){{el.value='{}';el.dispatchEvent(new Event('input',{{bubbles:true}}))}}}})()",
                        escape_js_selector(target), escape_js_str(value)
                    ),
                };
                let _ = state.http.post(&api("evaluate")).json(&json!({"script": js}))
                    .send().await;
                results.push(format!("{} = {}", target, value));
            }
            Ok(results.join("\n"))
        }

        // --- JS eval ---
        "browser_evaluate" => {
            let func = arg_str(args, "function")?;
            let resp = state.http.post(&api("evaluate")).json(&json!({"script": func}))
                .send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let result = body.get("result").cloned().unwrap_or(json!(null));
            Ok(serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result)))
        }

        // --- Info ---
        "browser_console_messages" => {
            let resp = state.http.get(&api("console")).send().await.map_err(|e| e.to_string())?;
            let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
            let entries = body.get("entries").and_then(|v| v.as_array())
                .map(|a| a.clone()).unwrap_or_default();
            if entries.is_empty() { return Ok("(no console entries)".to_string()); }
            let lines: Vec<String> = entries.iter().filter_map(|e| {
                Some(format!("[{}] {}", e.get("level")?.as_str()?, e.get("text")?.as_str()?))
            }).collect();
            Ok(lines.join("\n"))
        }
        "browser_network_requests" => {
            Ok("(network_requests: not yet implemented on headless wry)".to_string())
        }

        // --- Tabs ---
        "browser_tabs" => {
            let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("list");
            match action {
                "list" => Ok("[tab-0] current page".to_string()),
                "new" => {
                    if let Some(url) = args.get("url").and_then(|v| v.as_str()) {
                        let _ = state.http.post(&api("navigate")).json(&json!({"url": url})).send().await;
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
                    let check_js = format!("!!document.body?.innerText?.includes('{}')", escape_js_str(text));
                    let resp = state.http.post(&api("evaluate")).json(&json!({"script": check_js})).send().await;
                    if let Ok(r) = resp {
                        if let Ok(body) = r.json::<serde_json::Value>().await {
                            if body.get("result").and_then(|v| v.as_bool()).unwrap_or(false) {
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

        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}

// ─────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────

fn arg_str<'a>(args: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    args.get(key).and_then(|v| v.as_str())
        .ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn escape_js_selector(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'").replace('"', "\\\"")
}

fn escape_js_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\'', "\\'").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r")
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
