//! MCP (Model Context Protocol) server for tairitsu-packager.
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
//! configuration in `.opencode.jsonc` or similar editors.

use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";
const TOOL_LIST_PREFIX: &str = "tairitsu/";

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

    eprintln!("[tairitsu-mcp] Connected to daemon at {}", state.base_url);

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
// Tool definitions
// ─────────────────────────────────────────────────────

fn get_tool_list() -> Vec<serde_json::Value> {
    vec![
        make_tool_def(
            "browser_navigate",
            "Navigate browser to a URL",
            r#"{"type":"object","properties":{"url":{"type":"string","description":"URL to navigate to"}},"required":["url"]}"#,
        ),
        make_tool_def(
            "browser_snapshot",
            "Get accessibility snapshot of current page (DOM tree with roles and names)",
            "{}",
        ),
        make_tool_def(
            "browser_click",
            "Click an element matching a CSS selector",
            r#"{"type":"object","properties":{"selector":{"type":"string","description":"CSS selector for element to click"}},"required":["selector"]}"#,
        ),
        make_tool_def(
            "browser_evaluate",
            "Evaluate JavaScript expression in the page context",
            r#"{"type":"object","properties":{"script":{"type":"string","description":"JavaScript expression to evaluate"}},"required":["script"]}"#,
        ),
        make_tool_def(
            "browser_screenshot",
            "Take a PNG screenshot of the current page (returns base64-encoded image)",
            "{}",
        ),
        make_tool_def(
            "browser_console",
            "Get recent console log entries from the page",
            "{}",
        ),
        make_tool_def(
            "browser_status",
            "Get debug API status (connected, URL, etc.)",
            "{}",
        ),
    ]
}

fn make_tool_def(name: &str, description: &str, input_schema: &str) -> serde_json::Value {
    let schema: serde_json::Value =
        serde_json::from_str(input_schema).unwrap_or(serde_json::json!({}));
    serde_json::json!({
        "name": format!("{}{}", TOOL_LIST_PREFIX, name),
        "description": description,
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

                match serde_json::from_str::<JsonRpcRequest>(&trimmed) {
                    Ok(req) => {
                        let resp = handle_request(&state, req).await;
                        let output = serde_json::to_string(&resp).unwrap_or_default();
                        writeln!(stdout, "{}", output).ok();
                        stdout.flush().ok();
                    }
                    Err(e) => {
                        let resp = JsonRpcResponse::err(None, -32700, format!("Parse error: {}", e));
                        let output = serde_json::to_string(&resp).unwrap_or_default();
                        writeln!(stdout, "{}", output).ok();
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
        "notifications/initialized" => {
            JsonRpcResponse::ok(None, serde_json::Value::Null)
        }
        "tools/list" => handle_tools_list(req.id),
        "tools/call" => handle_tools_call(state, req.id, req.params).await,
        "ping" => JsonRpcResponse::ok(req.id, serde_json::json!({})),
        _ => JsonRpcResponse::err(
            req.id,
            -32601,
            format!("Method not found: {}", req.method),
        ),
    }
}

fn handle_initialize(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(
        id,
        serde_json::json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "tairitsu-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn handle_tools_list(id: Option<serde_json::Value>) -> JsonRpcResponse {
    JsonRpcResponse::ok(id, serde_json::json!({ "tools": get_tool_list() }))
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
        .unwrap_or(serde_json::json!({}));

    let tool_name = name.strip_prefix(TOOL_LIST_PREFIX).unwrap_or(name);

    match invoke_tool(state, tool_name, &arguments).await {
        Ok(result) => JsonRpcResponse::ok(
            id,
            serde_json::json!({
                "content": [{"type": "text", "text": result}],
                "isError": false
            }),
        ),
        Err(err_msg) => JsonRpcResponse::ok(
            id,
            serde_json::json!({
                "content": [{"type": "text", "text": err_msg}],
                "isError": true
            }),
        ),
    }
}

async fn invoke_tool(
    state: &McpState,
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<String, String> {
    let base = &state.base_url;
    let api_base = format!("{}/__tairitsu_debug", base);

    match tool_name {
        "browser_navigate" => {
            let url = args
                .get("url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: url".to_string())?;
            let resp = state
                .http
                .post(&format!("{}/navigate", api_base))
                .json(&serde_json::json!({"url": url}))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            Ok(format!(
                "Navigated to {} — ok={}",
                url,
                body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false)
            ))
        }
        "browser_snapshot" => {
            let resp = state
                .http
                .get(&format!("{}/snapshot", api_base))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            let snapshot = body
                .get("snapshot")
                .and_then(|v| v.as_str())
                .unwrap_or("(empty snapshot)");
            let title = body
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let url = body
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(format!(
                "Page: {}\nURL: {}\n\n{}",
                title, url, snapshot
            ))
        }
        "browser_click" => {
            let selector = args
                .get("selector")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: selector".to_string())?;
            let resp = state
                .http
                .post(&format!("{}/click", api_base))
                .json(&serde_json::json!({"selector": selector}))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            let ok = body
                .get("ok")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if ok {
                Ok(format!("Clicked: {}", selector))
            } else {
                Err(format!("Click failed on: {}", selector))
            }
        }
        "browser_evaluate" => {
            let script = args
                .get("script")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "Missing required parameter: script".to_string())?;
            let resp = state
                .http
                .post(&format!("{}/evaluate", api_base))
                .json(&serde_json::json!({"script": script}))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            let result = body.get("result").cloned().unwrap_or(serde_json::Value::Null);
            Ok(format!(
                "Result:\n{}",
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| format!("{:?}", result))
            ))
        }
        "browser_screenshot" => {
            let resp = state
                .http
                .get(&format!("{}/screenshot", api_base))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            if body.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                Ok("Screenshot captured successfully (base64 PNG data returned)".to_string())
            } else {
                let err = body
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error");
                Err(format!("Screenshot failed: {}", err))
            }
        }
        "browser_console" => {
            let resp = state
                .http
                .get(&format!("{}/console", api_base))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            let entries = body
                .get("entries")
                .and_then(|v| v.as_array())
                .map(|a| a.clone())
                .unwrap_or_default();
            if entries.is_empty() {
                return Ok("(no console entries)".to_string());
            }
            let lines: Vec<String> = entries
                .iter()
                .filter_map(|e| {
                    let level = e.get("level")?.as_str()?;
                    let text = e.get("text")?.as_str()?;
                    Some(format!("[{}] {}", level.to_lowercase(), text))
                })
                .collect();
            Ok(lines.join("\n"))
        }
        "browser_status" => {
            let resp = state
                .http
                .get(&format!("{}/status", api_base))
                .send()
                .await
                .map_err(|e| format!("HTTP error: {}", e))?;
            let body: serde_json::Value = resp.json().await.map_err(|e| format!("{}", e))?;
            Ok(serde_json::to_string_pretty(&body).unwrap_or_else(|_| format!("{:?}", body)))
        }
        _ => Err(format!("Unknown tool: {}", tool_name)),
    }
}
