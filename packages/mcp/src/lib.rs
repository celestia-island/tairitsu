#[cfg(feature = "vtty")]
mod vtty;

use anyhow::Result;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;

struct Server {
    base_url: Arc<RwLock<String>>,
    http: reqwest::Client,
    #[cfg(feature = "vtty")]
    vtty: Arc<vtty::VttyManager>,
}

impl Server {
    async fn api_async(&self, path: &str) -> String {
        let base = self.base_url.read().await.clone();
        format!("{}/{}", base, path)
    }

    async fn ensure_daemon(&self) -> Result<String, McpError> {
        {
            let url = self.base_url.read().await.clone();
            if !url.is_empty() {
                return Ok(url);
            }
        }
        let resolved = resolve_daemon_url().await.unwrap_or_default();
        if resolved.is_empty() {
            return Err(McpError::internal_error(
                "Browser tools require a running daemon. Start with: tairitsu dev --daemon --debug",
                None,
            ));
        }
        *self.base_url.write().await = resolved.clone();
        Ok(resolved)
    }

    fn tool_result(text: impl Into<String>) -> CallToolResult {
        CallToolResult::success(vec![Content::text(text)])
    }

    async fn http_post(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let url = self.api_async(path).await;
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("HTTP request failed: {e}"), None))?;
        let status = resp.status();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| McpError::internal_error(format!("Bad response body: {e}"), None))?;
        if !status.is_success() {
            let msg = v
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error");
            return Err(McpError::internal_error(
                format!("daemon returned {status}: {msg}"),
                None,
            ));
        }
        Ok(v)
    }

    async fn http_get(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<serde_json::Value, McpError> {
        let url = self.api_async(path).await;
        let resp = self
            .http
            .get(&url)
            .query(query)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("HTTP request failed: {e}"), None))?;
        let status = resp.status();
        let v: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| McpError::internal_error(format!("Bad response body: {e}"), None))?;
        if !status.is_success() {
            let msg = v
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown error");
            return Err(McpError::internal_error(
                format!("daemon returned {status}: {msg}"),
                None,
            ));
        }
        Ok(v)
    }

    async fn http_post_fire_and_forget(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<(), McpError> {
        let url = self.api_async(path).await;
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| McpError::internal_error(format!("HTTP request failed: {e}"), None))?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(McpError::internal_error(
                format!("daemon returned {status}: {text}"),
                None,
            ));
        }
        Ok(())
    }
}

// ── Tool argument structs ────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserNavigateArgs {
    url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SnapshotArgs {
    target: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ScreenshotArgs {
    element: Option<String>,
    #[serde(rename = "fullPage")]
    full_page: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ClickArgs {
    target: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct TypeArgs {
    submit: Option<bool>,
    target: String,
    text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct PressKeyArgs {
    key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EvaluateArgs {
    function: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ConsoleMessagesArgs {
    level: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct BrowserResizeArgs {
    width: u32,
    height: u32,
}

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
        let _ = self.ensure_daemon().await;
        self.http_post_fire_and_forget("navigate", json!({"url": args.url}))
            .await?;
        Ok(Self::tool_result(format!("Navigated to {}", args.url)))
    }

    #[tool(description = "Go back to the previous page")]
    async fn browser_navigate_back(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Ok(Self::tool_result(
            "(navigate-back: not yet supported via debug API)",
        ))
    }

    #[tool(description = "Go forward to the next page")]
    async fn browser_navigate_forward(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        Ok(Self::tool_result(
            "(navigate-forward: not yet supported via debug API)",
        ))
    }

    #[tool(
        description = "Capture accessibility snapshot of the current page (DOM tree with roles, names, text). Better than screenshot for understanding page structure."
    )]
    async fn browser_snapshot(
        &self,
        Parameters(args): Parameters<SnapshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        let query: Vec<(&str, &str)> = args
            .target
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| vec![("selector", s)])
            .unwrap_or_default();
        let v = self.http_get("a11y", &query).await?;
        Ok(Self::tool_result(
            v.get("data")
                .map(|d| serde_json::to_string(d).unwrap_or_else(|_| "{}".into()))
                .unwrap_or_else(|| "{}".into()),
        ))
    }

    #[tool(
        description = "Take a screenshot of the current viewport as PNG (returns base64 data URL)"
    )]
    async fn browser_screenshot(
        &self,
        Parameters(args): Parameters<ScreenshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        let mut body = json!({});
        if let Some(el) = &args.element {
            body["selector"] = json!(el);
        }
        if let Some(fp) = args.full_page {
            body["full_page"] = json!(fp);
        }
        let v = self.http_post("screenshot", body).await?;
        let ok = v.get("ok").and_then(|s| s.as_bool()).unwrap_or(false);
        if ok {
            let data = v
                .get("data")
                .and_then(|d| {
                    d.as_str()
                        .map(|s| s.to_string())
                        .or_else(|| {
                            d.get("data")
                                .and_then(|dd| dd.as_str())
                                .map(|s| s.to_string())
                        })
                        .or_else(|| {
                            d.as_object()
                                .map(|_| serde_json::to_string(d).unwrap_or_default())
                        })
                })
                .unwrap_or_default();
            let mime = v
                .get("data")
                .and_then(|d| d.get("mime_type"))
                .and_then(|m| m.as_str())
                .unwrap_or("image/png");
            let data_url = if data.starts_with("data:") {
                data
            } else {
                format!("data:{mime};base64,{data}")
            };
            Ok(CallToolResult::success(vec![Content::text(data_url)]))
        } else {
            let err = v
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown")
                .to_string();
            Err(McpError::internal_error(err, None))
        }
    }

    #[tool(description = "Click an element by CSS selector or reference from snapshot")]
    async fn browser_click(
        &self,
        Parameters(args): Parameters<ClickArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        self.http_post_fire_and_forget("click", json!({"selector": args.target}))
            .await?;
        Ok(Self::tool_result(format!("Clicked: {}", args.target)))
    }

    #[tool(description = "Type text into an editable element (input, textarea, contenteditable)")]
    async fn browser_type(
        &self,
        Parameters(args): Parameters<TypeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        self.http_post_fire_and_forget(
            "type",
            json!({
                "selector": args.target,
                "text": args.text,
                "clear_first": false,
                "submit": args.submit.unwrap_or(false)
            }),
        )
        .await?;
        Ok(Self::tool_result(format!("Typed: {}", args.text)))
    }

    #[tool(description = "Press a keyboard key (Enter, Tab, Escape, ArrowUp, etc.)")]
    async fn browser_press_key(
        &self,
        Parameters(args): Parameters<PressKeyArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        self.http_post_fire_and_forget("press", json!({"key": args.key}))
            .await?;
        Ok(Self::tool_result(format!("Pressed: {}", args.key)))
    }

    #[tool(description = "Evaluate JavaScript expression in the page context and return result")]
    async fn browser_evaluate(
        &self,
        Parameters(args): Parameters<EvaluateArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        let v = self
            .http_post("evaluate", json!({"expression": args.function}))
            .await?;
        let result = v
            .get("data")
            .and_then(|d| {
                d.as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        d.get("result")
                            .and_then(|r| r.as_str())
                            .map(|s| s.to_string())
                    })
                    .or_else(|| Some(serde_json::to_string(d).unwrap_or_default()))
            })
            .unwrap_or_default();
        Ok(Self::tool_result(result))
    }

    #[tool(description = "Get console log entries (error/warning/info/debug) from the page")]
    async fn browser_console_messages(
        &self,
        Parameters(args): Parameters<ConsoleMessagesArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        let level = args.level.as_deref().unwrap_or("");
        let v = self.http_get("console", &[("level", level)]).await?;
        Ok(Self::tool_result(v.to_string()))
    }

    #[tool(description = "Resize the browser window")]
    async fn browser_resize(
        &self,
        Parameters(args): Parameters<BrowserResizeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let _ = self.ensure_daemon().await;
        self.http_post_fire_and_forget(
            "resize",
            json!({"width": args.width, "height": args.height}),
        )
        .await?;
        Ok(Self::tool_result(format!(
            "Resized to {}x{}",
            args.width, args.height
        )))
    }

    // ── VTty tools ─────────────────────────────────────

    #[cfg(feature = "vtty")]
    #[tool(description = "Launch a command in a virtual terminal session")]
    async fn vtty_launch(
        &self,
        Parameters(args): Parameters<VttyLaunchArgs>,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let resolved_cwd = match args.cwd.as_deref() {
            Some(c) => Some(c.to_string()),
            None => resolve_default_cwd(&context).await,
        };
        let info = self
            .vtty
            .launch(
                &args.command,
                args.cols.unwrap_or(120) as u16,
                args.rows.unwrap_or(40) as u16,
                args.env.as_deref().unwrap_or(""),
                resolved_cwd.as_deref(),
                args.name.as_deref().unwrap_or(""),
            )
            .map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Kill a virtual terminal session")]
    async fn vtty_kill(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self
            .vtty
            .kill(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(
        description = "Send key sequences to a virtual terminal. Supports Enter, Tab, Escape, Backspace, Delete, Arrow keys, Home/End, PageUp/PageDown, F1-F12, Ctrl+X, Alt+X"
    )]
    async fn vtty_send_keys(
        &self,
        Parameters(args): Parameters<VttySendKeysArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        {
            let guard = session
                .lock()
                .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            guard
                .send_keys(&args.keys)
                .map_err(|e| McpError::internal_error(e, None))?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        {
            let guard = session
                .lock()
                .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            let _ = guard.read_and_update();
        }
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "keys": args.keys, "sent": true}).to_string(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Send text string to a virtual terminal")]
    async fn vtty_send_text(
        &self,
        Parameters(args): Parameters<VttySendTextArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        {
            let guard = session
                .lock()
                .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            guard
                .send_text(&args.text)
                .map_err(|e| McpError::internal_error(e, None))?;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        {
            let guard = session
                .lock()
                .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            let _ = guard.read_and_update();
        }
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "length": args.text.len(), "sent": true})
                .to_string(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(
        description = "Capture current terminal screen content as text (text-only models) and/or as a rendered PNG image (vision-capable models). \
        The 'format' parameter controls output: 'text' (default) returns plain text, 'image' returns a rendered PNG, 'both' returns both. \
        The 'theme' parameter sets the color scheme: solarized-dark (default), solarized-light, one-half-dark, one-half-light, ibm-5153."
    )]
    async fn vtty_screenshot(
        &self,
        Parameters(args): Parameters<VttyScreenshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        let fmt = args.format.as_deref().unwrap_or("text");
        let theme = args.theme.as_deref().unwrap_or("solarized-dark");

        let (text, alive, rows, cols) = {
            let guard = session
                .lock()
                .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
            (guard.screenshot(), guard.is_alive(), guard.rows, guard.cols)
        };

        match fmt {
            "text" => Ok(Self::tool_result(
                json!({
                    "session_id": args.session_id,
                    "alive": alive,
                    "rows": rows,
                    "cols": cols,
                    "text": text
                })
                .to_string(),
            )),
            #[cfg(feature = "vtty-visual")]
            "image" => {
                let guard = session
                    .lock()
                    .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let png_data = guard
                    .visual_screenshot(theme)
                    .map_err(|e| McpError::internal_error(e, None))?;
                let b64 = vtty::render::encode_base64(&png_data);
                Ok(CallToolResult::success(vec![Content::image(
                    b64,
                    "image/png",
                )]))
            }
            #[cfg(feature = "vtty-visual")]
            "both" => {
                let guard = session
                    .lock()
                    .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let png_data = guard
                    .visual_screenshot(theme)
                    .map_err(|e| McpError::internal_error(e, None))?;
                let b64 = vtty::render::encode_base64(&png_data);
                Ok(CallToolResult::success(vec![
                    Content::text(
                        json!({
                            "session_id": args.session_id,
                            "alive": alive,
                            "rows": rows,
                            "cols": cols,
                            "text": text
                        })
                        .to_string(),
                    ),
                    Content::image(b64, "image/png"),
                ]))
            }
            _ => Ok(Self::tool_result(
                json!({
                    "session_id": args.session_id,
                    "alive": alive,
                    "rows": rows,
                    "cols": cols,
                    "text": text
                })
                .to_string(),
            )),
        }
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Wait for duration or until text appears on screen")]
    async fn vtty_wait(
        &self,
        Parameters(args): Parameters<VttyWaitArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        let secs = args.seconds.unwrap_or(5.0);
        let pattern = args.pattern.unwrap_or_default();
        if !pattern.is_empty() {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_secs_f64(secs.min(1800.0));
            let mut found = false;
            while std::time::Instant::now() < deadline {
                let alive = {
                    let guard = session
                        .lock()
                        .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                    if !guard.is_alive() {
                        false
                    } else {
                        let _ = guard.read_and_update();
                        let f = !guard.find_text(&pattern).is_empty();
                        if f {
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
            Ok(Self::tool_result(json!({"session_id": args.session_id, "pattern": pattern, "found": found, "alive": alive}).to_string()))
        } else {
            let wait_secs = secs.min(1800.0) as u64;
            let mut alive = true;
            for _ in 0..(wait_secs * 20) {
                alive = {
                    let guard = session
                        .lock()
                        .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
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
            if alive {
                let guard = session
                    .lock()
                    .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                let _ = guard.read_and_update();
                alive = guard.is_alive();
            }
            Ok(Self::tool_result(
                json!({"session_id": args.session_id, "seconds_waited": secs, "alive": alive})
                    .to_string(),
            ))
        }
    }

    #[cfg(feature = "vtty")]
    #[tool(
        description = "Wait until a VTty session has screen output (useful after vtty_launch for slow-starting commands). Returns immediately if output is already present."
    )]
    async fn vtty_ready(
        &self,
        Parameters(args): Parameters<VttyReadyArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let timeout_ms = args.timeout_ms.unwrap_or(30000);
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut ready = false;
        while std::time::Instant::now() < deadline {
            {
                let guard = session
                    .lock()
                    .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
                if guard.has_output() {
                    ready = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "ready": ready}).to_string(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(
        description = "Get the scrollback buffer (history) of a virtual terminal session, including current screen content"
    )]
    async fn vtty_scrollback(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        let guard = session
            .lock()
            .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
        let text = guard.scrollback();
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "text": text}).to_string(),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "Resize a virtual terminal")]
    async fn vtty_resize(
        &self,
        Parameters(args): Parameters<VttyResizeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let session = self
            .vtty
            .get(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        let guard = session
            .lock()
            .map_err(|e| McpError::internal_error(format!("{}", e), None))?;
        let old = (guard.cols, guard.rows);
        guard
            .resize(args.cols as u16, args.rows as u16)
            .map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(json!({"session_id": args.session_id, "old": {"cols": old.0, "rows": old.1}, "new": {"cols": args.cols, "rows": args.rows}}).to_string()))
    }

    #[cfg(feature = "vtty")]
    #[tool(description = "List all active virtual terminal sessions")]
    async fn vtty_list(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let sessions = self.vtty.list();
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| "[]".to_string()),
        ))
    }

    #[cfg(feature = "vtty")]
    #[tool(
        description = "Check if a VTty session's child process is still alive and refresh screen state"
    )]
    async fn vtty_ping(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self
            .vtty
            .ping(&args.session_id)
            .map_err(|e| McpError::internal_error(e, None))?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }
}

// ── ServerHandler ────────────────────────────────────

#[tool_handler(router = Server::tool_router())]
impl ServerHandler for Server {}

// ── default CWD resolution ──────────────────────────

#[cfg(feature = "vtty")]
async fn resolve_default_cwd(context: &RequestContext<RoleServer>) -> Option<String> {
    if let Ok(root) = std::env::var("TAIRITSU_PROJECT_ROOT") {
        if !root.is_empty() {
            return Some(root);
        }
    }

    if let Some(info) = context.peer.peer_info() {
        if info.capabilities.roots.is_some() {
            if let Ok(result) = context.peer.list_roots().await {
                if let Some(root) = result.roots.first() {
                    let uri = &root.uri;
                    let path = if let Some(p) = uri.strip_prefix("file://") {
                        p.to_string()
                    } else if let Some(p) = uri.strip_prefix("file:") {
                        p.to_string()
                    } else {
                        uri.clone()
                    };
                    if !path.is_empty() {
                        return Some(path);
                    }
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        return Some(cwd.to_string_lossy().to_string());
    }

    None
}

// ── daemon resolution ───────────────────────────────

mod daemon {
    use anyhow::anyhow;
    use std::path::PathBuf;

    pub(super) async fn resolve_daemon_url() -> anyhow::Result<String> {
        if let Ok(url) = std::env::var("TAIRITSU_DAEMON_URL") {
            if !url.is_empty() {
                return Ok(url);
            }
        }

        let priority_dirs: Vec<PathBuf> = {
            let mut v = Vec::new();
            if let Ok(root) = std::env::var("TAIRITSU_PROJECT_ROOT") {
                let p = PathBuf::from(&root);
                v.push(p.join("target"));
            }
            if let Ok(cwd) = std::env::current_dir() {
                v.push(cwd.join("target"));
                let mut dir = cwd.clone();
                for _ in 0..8 {
                    if dir.join("Cargo.toml").exists() {
                        v.push(dir.join("target"));
                    }
                    if !dir.pop() {
                        break;
                    }
                }
            }
            v
        };
        if let Some((_port, debug_port, _)) = try_read_ready_port_from_candidates(&priority_dirs) {
            if let Some(dp) = debug_port {
                return Ok(format!("http://localhost:{dp}"));
            }
            return Err(anyhow!(
                "Daemon found but debug API not enabled. Start with: tairitsu dev --daemon --debug"
            ));
        }

        let searched = search_project_roots_fallback();
        if let Some((_port, debug_port, _)) = try_read_ready_port_from_candidates(&searched) {
            if let Some(dp) = debug_port {
                return Ok(format!("http://localhost:{dp}"));
            }
            return Err(anyhow!(
                "Daemon found but debug API not enabled. Start with: tairitsu dev --daemon --debug"
            ));
        }
        Err(anyhow!("No running tairitsu daemon found"))
    }

    fn search_project_roots_fallback() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            add_target_tree(&mut candidates, &cwd, 2);
        }
        if let Ok(root) = std::env::var("TAIRITSU_PROJECT_ROOT") {
            let root_path = PathBuf::from(&root);
            add_target_tree(&mut candidates, &root_path, 2);
        }
        for scan_dir in std::env::var("HOME")
            .ok()
            .map(|h| vec![PathBuf::from("/mnt/sdb1"), PathBuf::from(h)])
            .unwrap_or_default()
        {
            if let Ok(entries) = std::fs::read_dir(&scan_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        candidates.push(p.join("target"));
                    }
                }
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
                candidates.push(parent.join("target"));
            }
        }
        candidates.dedup();
        candidates
    }

    fn add_target_tree(candidates: &mut Vec<PathBuf>, base: &PathBuf, depth: u32) {
        if depth == 0 {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    candidates.push(p.join("target"));
                    add_target_tree(candidates, &p, depth - 1);
                }
            }
        }
    }

    fn try_read_ready_port_from_candidates(
        dirs: &[PathBuf],
    ) -> Option<(u16, Option<u16>, PathBuf)> {
        for dir in dirs {
            let ready_path = dir.join("tairitsu-packager.ready");
            if let Ok(content) = std::fs::read_to_string(&ready_path) {
                let trimmed = content.trim();
                if let Some(rest) = trimmed.strip_prefix("ready:") {
                    let mut parts = rest.splitn(2, ':');
                    if let Some(port_str) = parts.next() {
                        if let Ok(port) = port_str.parse::<u16>() {
                            let debug_port = parts.next().and_then(|s| s.parse().ok());
                            return Some((port, debug_port, ready_path));
                        }
                    }
                } else if trimmed == "ready" {
                    return Some((3000, None, ready_path));
                }
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
    let base_url = Arc::new(RwLock::new(String::new()));

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
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default(),
        #[cfg(feature = "vtty")]
        vtty: Arc::new(vtty::VttyManager::new()),
    };

    let transport = rmcp::transport::stdio();
    let server_handle = server.serve(transport).await?;
    server_handle.waiting().await?;

    Ok(())
}
