//! Standalone Tairitsu MCP server — browser automation + VTty for AI coding assistants.
//! Built on `rmcp` — the official Rust MCP SDK.

use anyhow::Result;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
    handler::server::wrapper::Parameters,
    model::*,
    service::RequestContext,
    tool, tool_router, tool_handler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

#[cfg(feature = "vtty")]
mod vtty;

// ── Server state ────────────────────────────────────

#[derive(Clone)]
pub struct Server {
    base_url: std::sync::Arc<tokio::sync::RwLock<String>>,
    http: reqwest::Client,
    #[cfg(feature = "vtty")]
    vtty: std::sync::Arc<vtty::VttyManager>,
}

impl Server {
    fn api(&self, path: &str) -> String {
        let base = self.base_url.blocking_read();
        format!("{}/__tairitsu_debug/{}", base, path)
    }

    fn check_daemon(&self) -> Result<String, String> {
        let url = self.base_url.blocking_read().clone();
        if url.is_empty() {
            Err("Browser tools require a running daemon. Start with: tairitsu dev --daemon".into())
        } else {
            Ok(url)
        }
    }

    fn tool_result(text: impl Into<String>) -> CallToolResult {
        CallToolResult::success(vec![Content::text(text)])
    }
}

// ── Tool argument structs ────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserNavigateArgs { url: String }

#[derive(Debug, Deserialize, JsonSchema)]
struct SnapshotArgs { target: Option<String> }

#[derive(Debug, Deserialize, JsonSchema)]
struct ScreenshotArgs {
    element: Option<String>,
    #[serde(rename = "fullPage")]
    full_page: Option<bool>,
    #[serde(rename = "type")]
    image_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct ClickArgs {
    element: Option<String>,
    target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct TypeArgs {
    element: Option<String>,
    submit: Option<bool>,
    target: String,
    text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PressKeyArgs { key: String }

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct HoverArgs {
    element: Option<String>,
    target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct SelectOptionArgs {
    element: Option<String>,
    target: String,
    values: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct FillFormField {
    element: Option<String>,
    name: String,
    target: String,
    #[serde(rename = "type")]
    field_type: String,
    value: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FillFormArgs { fields: Vec<FillFormField> }

#[derive(Debug, Deserialize, JsonSchema)]
#[allow(dead_code)]
struct EvaluateArgs {
    element: Option<String>,
    function: String,
    target: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ConsoleMessagesArgs {
    all: Option<bool>,
    level: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct NetworkRequestsArgs {
    filter: Option<String>,
    #[serde(rename = "static")]
    is_static: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserTabsArgs {
    action: Option<String>,
    index: Option<u32>,
    url: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct WaitForArgs {
    text: Option<String>,
    #[serde(rename = "textGone")]
    text_gone: Option<String>,
    time: Option<f64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserResizeArgs {
    width: u32,
    height: u32,
}

// ── VTty tool args ───────────────────────────────────

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttyLaunchArgs {
    command: String,
    cols: Option<u64>,
    rows: Option<u64>,
    env: Option<String>,
    cwd: Option<String>,
    name: Option<String>,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttySessionArgs {
    session_id: String,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttyScreenshotArgs {
    session_id: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    theme: Option<String>,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttySendKeysArgs {
    session_id: String,
    keys: String,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttySendTextArgs {
    session_id: String,
    text: String,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttyWaitArgs {
    session_id: String,
    seconds: Option<f64>,
    pattern: Option<String>,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttyReadyArgs {
    session_id: String,
    timeout_ms: Option<u64>,
}

#[cfg(feature = "vtty")]
#[derive(Debug, Deserialize, JsonSchema)]
struct VttyResizeArgs {
    session_id: String,
    cols: u64,
    rows: u64,
}

// ── Browser tools ────────────────────────────────────

#[tool_router]
impl Server {
    #[tool(description = "Navigate to a URL")]
    async fn browser_navigate(
        &self,
        Parameters(args): Parameters<BrowserNavigateArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let _resp = self.http.post(self.api("navigate")).json(&json!({"url": args.url})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Navigated to {}", args.url)))
    }

    #[tool(description = "Go back to the previous page")]
    async fn browser_navigate_back(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Ok(Self::tool_result("(navigate-back: not yet implemented on headless wry)"))
    }

    #[tool(description = "Go forward to the next page")]
    async fn browser_navigate_forward(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Ok(Self::tool_result("(navigate-forward: not yet implemented on headless wry)"))
    }

    #[tool(description = "Capture accessibility snapshot of the current page (DOM tree with roles, names, text). Better than screenshot for understanding page structure.")]
    async fn browser_snapshot(
        &self,
        Parameters(args): Parameters<SnapshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(t) = &args.target { body["selector"] = json!(t); }
        let resp = self.http.post(self.api("snapshot")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let text = resp.json::<serde_json::Value>().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let out = text.get("snapshot").and_then(|v| v.as_str()).unwrap_or("{}");
        Ok(Self::tool_result(out.to_string()))
    }

    #[tool(description = "Take a screenshot of the current viewport as PNG (returns base64 data URL)")]
    async fn browser_screenshot(
        &self,
        Parameters(args): Parameters<ScreenshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(el) = &args.element { body["selector"] = json!(el); }
        if let Some(fp) = args.full_page { body["fullPage"] = json!(fp); }
        if let Some(t) = &args.image_type { body["type"] = json!(t); }
        let resp = self.http.post(self.api("screenshot")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let v: serde_json::Value = resp.json().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let success = v.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
        let data = v.get("data").and_then(|d| d.as_str()).unwrap_or("").to_string();
        let err = v.get("error").and_then(|e| e.as_str()).unwrap_or("unknown").to_string();
        if success {
            Ok(Self::tool_result(data))
        } else {
            Err(McpError::internal_error(err, None))
        }
    }

    #[tool(description = "Click an element by CSS selector or reference from snapshot")]
    async fn browser_click(
        &self,
        Parameters(args): Parameters<ClickArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("click")).json(&json!({"selector": args.target})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Clicked: {}", args.target)))
    }

    #[tool(description = "Type text into an editable element (input, textarea, contenteditable)")]
    async fn browser_type(
        &self,
        Parameters(args): Parameters<TypeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let js = format!("const e=document.querySelector('{}');if(e){{e.focus();e.value='{}';e.dispatchEvent(new Event('input',{{bubbles:true}}));}}{}", args.target, args.text.replace('\'', "\\'"), if args.submit.unwrap_or(false) { "e.form?.submit();" } else { "" });
        self.http.post(self.api("evaluate")).json(&json!({"function": js})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Typed: {}", args.text)))
    }

    #[tool(description = "Press a keyboard key (Enter, Tab, Escape, ArrowUp, etc.)")]
    async fn browser_press_key(
        &self,
        Parameters(args): Parameters<PressKeyArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("press-key")).json(&json!({"key": args.key})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Pressed: {}", args.key)))
    }

    #[tool(description = "Hover mouse over an element (triggers tooltips, dropdowns, etc.)")]
    async fn browser_hover(
        &self,
        Parameters(args): Parameters<HoverArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("hover")).json(&json!({"selector": args.target})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Hovered: {}", args.target)))
    }

    #[tool(description = "Select option(s) in a <select> dropdown")]
    async fn browser_select_option(
        &self,
        Parameters(args): Parameters<SelectOptionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("select-option")).json(&json!({"selector": args.target, "values": args.values})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Selected in: {}", args.target)))
    }

    #[tool(description = "Fill multiple form fields at once")]
    async fn browser_fill_form(
        &self,
        Parameters(args): Parameters<FillFormArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let v: Vec<_> = args.fields.iter().map(|f| json!({"target": f.target, "name": f.name, "type": f.field_type, "value": f.value})).collect();
        self.http.post(self.api("fill-form")).json(&json!({"fields": v})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result("Form filled".to_string()))
    }

    #[tool(description = "Evaluate JavaScript expression in the page context and return result")]
    async fn browser_evaluate(
        &self,
        Parameters(args): Parameters<EvaluateArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({"function": args.function});
        if let Some(t) = &args.target { body["target"] = json!(t); }
        let resp = self.http.post(self.api("evaluate")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let v: serde_json::Value = resp.json().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(v.get("result").and_then(|r| r.as_str()).unwrap_or("").to_string()))
    }

    #[tool(description = "Get console log entries (error/warning/info/debug) from the page")]
    async fn browser_console_messages(
        &self,
        Parameters(args): Parameters<ConsoleMessagesArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(a) = args.all { body["all"] = json!(a); }
        if let Some(l) = &args.level { body["level"] = json!(l); }
        let resp = self.http.post(self.api("console-messages")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let v: serde_json::Value = resp.json().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(v.to_string()))
    }

    #[tool(description = "List HTTP network requests made by the page since last navigation")]
    async fn browser_network_requests(
        &self,
        Parameters(args): Parameters<NetworkRequestsArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(f) = &args.filter { body["filter"] = json!(f); }
        if let Some(s) = args.is_static { body["static"] = json!(s); }
        let resp = self.http.post(self.api("network-requests")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let v: serde_json::Value = resp.json().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(v.to_string()))
    }

    #[tool(description = "List, create, close, or switch browser tabs")]
    async fn browser_tabs(
        &self,
        Parameters(args): Parameters<BrowserTabsArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(a) = &args.action { body["action"] = json!(a); }
        if let Some(i) = args.index { body["index"] = json!(i); }
        if let Some(u) = &args.url { body["url"] = json!(u); }
        let resp = self.http.post(self.api("tabs")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        let v: serde_json::Value = resp.json().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(v.to_string()))
    }

    #[tool(description = "Wait for a condition: time (seconds), text appearance, or text disappearance")]
    async fn browser_wait_for(
        &self,
        Parameters(args): Parameters<WaitForArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        let mut body = json!({});
        if let Some(t) = &args.text { body["text"] = json!(t); }
        if let Some(tg) = &args.text_gone { body["textGone"] = json!(tg); }
        if let Some(t) = args.time { body["time"] = json!(t); }
        self.http.post(self.api("wait-for")).json(&body).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result("Wait condition satisfied".to_string()))
    }

    #[tool(description = "Close the current tab or entire browser session")]
    async fn browser_close(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("close")).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result("Browser closed".to_string()))
    }

    #[tool(description = "Resize the browser window")]
    async fn browser_resize(
        &self,
        Parameters(args): Parameters<BrowserResizeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.check_daemon().map_err(|e| McpError::internal_error(e, None))?;
        self.http.post(self.api("resize")).json(&json!({"width": args.width, "height": args.height})).send().await.map_err(|e| McpError::internal_error(e.to_string(), None))?;
        Ok(Self::tool_result(format!("Resized to {}x{}", args.width, args.height)))
    }

    // ── VTty tools ─────────────────────────────────────

    #[cfg(feature = "vtty")]
    #[tool(description = "Launch a command in a virtual terminal session")]
    async fn vtty_launch(
        &self,
        Parameters(args): Parameters<VttyLaunchArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self.vtty.launch(
            &args.command,
            args.cols.unwrap_or(120) as u16,
            args.rows.unwrap_or(40) as u16,
            args.env.as_deref().unwrap_or(""),
            args.cwd.as_deref(),
            args.name.as_deref().unwrap_or(""),
        ).map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(serde_json::to_string_pretty(&info).unwrap_or_default()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Kill a virtual terminal session")]
    async fn vtty_kill(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self.vtty.kill(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(serde_json::to_string_pretty(&info).unwrap_or_default()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Send key sequences to a virtual terminal. Supports Enter, Tab, Escape, Backspace, Delete, Arrow keys, Home/End, PageUp/PageDown, F1-F12, Ctrl+X, Alt+X")]
    async fn vtty_send_keys(
        &self,
        Parameters(args): Parameters<VttySendKeysArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        {
            let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            guard.send_keys(&args.keys).map_err(|e| McpError::internal_error(e, None))?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        {
            let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            let _ = guard.read_and_update();
        }
        Ok(Self::tool_result(json!({"session_id": args.session_id, "keys": args.keys, "sent": true}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Send text string to a virtual terminal")]
    async fn vtty_send_text(
        &self,
        Parameters(args): Parameters<VttySendTextArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        {
            let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            guard.send_text(&args.text).map_err(|e| McpError::internal_error(e, None))?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        {
            let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            let _ = guard.read_and_update();
        }
        Ok(Self::tool_result(json!({"session_id": args.session_id, "length": args.text.len(), "sent": true}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Capture current terminal screen content as text (text-only models) and/or as a rendered PNG image (vision-capable models). \
        The 'format' parameter controls output: 'text' (default) returns plain text, 'image' returns a rendered PNG, 'both' returns both. \
        The 'theme' parameter sets the color scheme: solarized-dark (default), solarized-light, one-half-dark, one-half-light, ibm-5153.")]
    async fn vtty_screenshot(
        &self,
        Parameters(args): Parameters<VttyScreenshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        let fmt = args.format.as_deref().unwrap_or("text");
        let theme = args.theme.as_deref().unwrap_or("solarized-dark");

        let (text, alive, rows, cols) = {
            let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            (guard.screenshot(), guard.is_alive(), guard.rows, guard.cols)
        };

        match fmt {
            "text" => {
                Ok(Self::tool_result(json!({
                    "session_id": args.session_id,
                    "alive": alive,
                    "rows": rows,
                    "cols": cols,
                    "text": text
                }).to_string()))
            }
            #[cfg(feature = "vtty-visual")]
            "image" => {
                let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let png_data = guard.visual_screenshot(theme).map_err(|e| McpError::internal_error(e, None))?;
                let b64 = vtty::render::encode_base64(&png_data);
                Ok(CallToolResult::success(vec![Content::image(b64, "image/png")]))
            }
            #[cfg(feature = "vtty-visual")]
            "both" => {
                let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let png_data = guard.visual_screenshot(theme).map_err(|e| McpError::internal_error(e, None))?;
                let b64 = vtty::render::encode_base64(&png_data);
                Ok(CallToolResult::success(vec![
                    Content::text(json!({
                        "session_id": args.session_id,
                        "alive": alive,
                        "rows": rows,
                        "cols": cols,
                        "text": text
                    }).to_string()),
                    Content::image(b64, "image/png"),
                ]))
            }
            _ => {
                Ok(Self::tool_result(json!({
                    "session_id": args.session_id,
                    "alive": alive,
                    "rows": rows,
                    "cols": cols,
                    "text": text
                }).to_string()))
            }
        }
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Wait for duration or until text appears on screen")]
    async fn vtty_wait(
        &self,
        Parameters(args): Parameters<VttyWaitArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        let secs = args.seconds.unwrap_or(5.0);
        let pattern = args.pattern.unwrap_or_default();
        if !pattern.is_empty() {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs_f64(secs.min(1800.0));
            let mut found = false;
            while std::time::Instant::now() < deadline {
                let alive = {
                    let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                    if !guard.is_alive() { false } else {
                        let _ = guard.read_and_update();
                        let f = !guard.find_text(&pattern).is_empty();
                        if f { found = true; }
                        guard.is_alive()
                    }
                };
                if found || !alive { break; }
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
            let alive = session.lock().map(|g| g.is_alive()).unwrap_or(false);
            Ok(Self::tool_result(json!({"session_id": args.session_id, "pattern": pattern, "found": found, "alive": alive}).to_string()))
        } else {
            let wait_secs = secs.min(1800.0) as u64;
            let mut alive = true;
            for _ in 0..(wait_secs * 20) {
                alive = {
                    let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                    if !guard.is_alive() { false } else { let _ = guard.read_and_update(); guard.is_alive() }
                };
                if !alive { break; }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            if alive {
                let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let _ = guard.read_and_update();
                alive = guard.is_alive();
            }
            Ok(Self::tool_result(json!({"session_id": args.session_id, "seconds_waited": secs, "alive": alive}).to_string()))
        }
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Wait until a VTty session has screen output (useful after vtty_launch for slow-starting commands). Returns immediately if output is already present.")]
    async fn vtty_ready(
        &self,
        Parameters(args): Parameters<VttyReadyArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let timeout_ms = args.timeout_ms.unwrap_or(30000);
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut ready = false;
        while std::time::Instant::now() < deadline {
            {
                let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                if guard.has_output() { ready = true; break; }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Ok(Self::tool_result(json!({"session_id": args.session_id, "ready": ready}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Get the scrollback buffer (history) of a virtual terminal session, including current screen content")]
    async fn vtty_scrollback(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
        let text = guard.scrollback();
        Ok(Self::tool_result(json!({"session_id": args.session_id, "text": text}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Resize a virtual terminal")]
    async fn vtty_resize(
        &self,
        Parameters(args): Parameters<VttyResizeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self.vtty.get(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        let guard = session.lock().map_err(|e| McpError::internal_error(format!("{}", e), None))?;
        let old = (guard.cols, guard.rows);
        guard.resize(args.cols as u16, args.rows as u16).map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(json!({"session_id": args.session_id, "old": {"cols": old.0, "rows": old.1}, "new": {"cols": args.cols, "rows": args.rows}}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "List all active virtual terminal sessions")]
    async fn vtty_list(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let sessions = self.vtty.list();
        Ok(Self::tool_result(serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| "[]".to_string())))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Check if a VTty session's child process is still alive and refresh screen state")]
    async fn vtty_ping(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self.vtty.ping(&args.session_id).map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(serde_json::to_string_pretty(&info).unwrap_or_default()))
    }
}

// ── ServerHandler ────────────────────────────────────

// ── ServerHandler ────────────────────────────────────

#[tool_handler(router = Server::tool_router())]
impl ServerHandler for Server {}

// ── daemon resolution (unchanged) ────────────────────

mod daemon {
    use anyhow::anyhow;
    use std::path::PathBuf;

    pub(super) async fn resolve_daemon_url() -> anyhow::Result<String> {
        if let Ok(url) = std::env::var("TAIRITSU_DAEMON_URL") && !url.is_empty() {
            tracing::debug!("[tairitsu-mcp] Using TAIRITSU_DAEMON_URL={}", url);
            return Ok(url);
        }
        let searched = search_project_roots();
        if let Some((port, _found_at)) = try_read_ready_port_from_candidates(&searched) {
            return Ok(format!("http://localhost:{}", port));
        }
        Err(anyhow!("No running tairitsu daemon found"))
    }

    fn search_project_roots() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("target"));
            let mut dir = cwd.clone();
            for _ in 0..5 {
                if dir.join("Cargo.toml").exists() { candidates.push(dir.join("target")); }
                if !dir.pop() { break; }
            }
        }
        if let Ok(root) = std::env::var("TAIRITSU_PROJECT_ROOT") {
            candidates.push(PathBuf::from(root).join("target"));
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(parent) = exe.parent().and_then(|p| p.parent()) {
                candidates.push(parent.join("target"));
            }
        candidates.dedup();
        candidates
    }

    fn try_read_ready_port_from_candidates(dirs: &[PathBuf]) -> Option<(u16, PathBuf)> {
        for dir in dirs {
            let ready_path = dir.join("tairitsu-packager.ready");
            if let Ok(content) = std::fs::read_to_string(&ready_path) {
                let trimmed = content.trim();
                if let Some(port_str) = trimmed.strip_prefix("ready:") {
                    if let Ok(port) = port_str.parse::<u16>() { return Some((port, ready_path)); }
                } else if trimmed == "ready" { return Some((3000, ready_path)); }
            }
        }
        None
    }
}

use daemon::resolve_daemon_url;

// ── public entry point ───────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    pub base_url: String,
}

pub async fn run(config: McpConfig) -> Result<()> {
    let base_url = std::sync::Arc::new(tokio::sync::RwLock::new(String::new()));

    // Resolve daemon URL in background (browser tools need this)
    let base_url_clone = base_url.clone();
    let url_from_config = config.base_url.clone();
    tokio::spawn(async move {
        let url = if !url_from_config.is_empty() {
            url_from_config
        } else {
            resolve_daemon_url().await.unwrap_or_default()
        };
        *base_url_clone.write().await = url;
    });

    let server = Server {
        base_url: base_url.clone(),
        http: reqwest::Client::new(),
        #[cfg(feature = "vtty")]
        vtty: std::sync::Arc::new(vtty::VttyManager::new()),
    };

    let transport = rmcp::transport::stdio();
    let server_handle = server.serve(transport).await?;
    server_handle.waiting().await?;

    Ok(())
}
