//! MCP (Model Context Protocol) server for Tairitsu.
//!
//! A thin wrapper: the **browser** tools proxy HTTP to a [shirabe] debug
//! server (the CDP engine extracted from the tairitsu packager), and the
//! **VTty** tools delegate in-process to the [kou] virtual-terminal engine
//! (PTY + VT100 + rendering). All the heavy lifting lives in those two
//! dedicated crates — this package is just the MCP tool wiring.
//!
//! [shirabe]: https://github.com/celestia-island/shirabe
//! [kou]: https://github.com/celestia-island/kou
//!
//! # Usage
//!
//! Point the browser tools at a running shirabe debug server (or the legacy
//! tairitsu daemon) and start the MCP server on stdio:
//!
//! ```ignore
//! SHIRABE_URL=http://localhost:3001 tairitsu-mcp
//! ```

use anyhow::Result;
use base64::Engine;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;

/// Font size used when rasterising VTty screenshots to PNG.
///
/// The renderer expects fonts loaded at `font_px * supersample`.  We target
/// a desktop-scale output (~1920 px wide for a 120-col terminal).
const FONT_PX: f32 = 32.0;
/// Supersample factor — render at this multiple then downscale with Lanczos3
/// for crisp, anti-aliased terminal glyphs.
const RENDER_SUPER: u32 = 3;

struct Server {
    base_url: Arc<RwLock<String>>,
    http: reqwest::Client,
    vtty: kou::VttyManager,
    fonts: Arc<kou::FontCache>,
}

impl Server {
    async fn api_async(&self, path: &str) -> String {
        let base = self.base_url.read().await.clone();
        format!("{}/{}", base, path)
    }

    async fn ensure_daemon(&self) -> Result<String, McpError> {
        let url = self.base_url.read().await.clone();
        if url.is_empty() {
            return Err(McpError::internal_error(
                "Browser tools require a running shirabe debug server (or tairitsu daemon). \
                 Set SHIRABE_URL / TAIRITSU_DAEMON_URL, or start: shirabe serve",
                None,
            ));
        }
        Ok(url)
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

    /// Rasterise a VTty screen to a base64-encoded PNG (for `image` / `both`
    /// screenshot modes), painted through `theme`.
    fn render_png(&self, screen: &kou::Screen, theme: &kou::Theme) -> Result<String, McpError> {
        let png =
            kou::render::render_png_supersampled(screen, &self.fonts, FONT_PX, RENDER_SUPER, theme)
                .map_err(|e| McpError::internal_error(format!("VTty render failed: {e}"), None))?;
        Ok(base64::engine::general_purpose::STANDARD.encode(&png))
    }

    /// Build the JSON object the `text` / `both` screenshot modes emit.
    fn screen_text_json(
        &self,
        session_id: &str,
        alive: bool,
        screen: &kou::Screen,
        text: &str,
    ) -> String {
        json!({
            "session_id": session_id,
            "alive": alive,
            "rows": screen.rows,
            "cols": screen.cols,
            "text": text,
        })
        .to_string()
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
struct DomQueryArgs {
    /// CSS selector for the element(s) to describe.
    selector: String,
    /// If set, return only this single attribute's value instead of the full element.
    attribute: Option<String>,
    /// If true, also include a default set of computed styles (display/color/dimensions/...).
    #[serde(rename = "computed")]
    computed: Option<bool>,
    /// If true, describe every match (not just the first) into `matches`.
    #[serde(rename = "all")]
    all: Option<bool>,
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

#[derive(Debug, Deserialize, JsonSchema)]
struct VttyLaunchArgs {
    command: String,
    cols: Option<u64>,
    rows: Option<u64>,
    env: Option<String>,
    cwd: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttySessionArgs {
    session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttyScreenshotArgs {
    session_id: String,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    theme: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttySendKeysArgs {
    session_id: String,
    keys: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttySendTextArgs {
    session_id: String,
    text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttyWaitArgs {
    session_id: String,
    seconds: Option<f64>,
    pattern: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttyReadyArgs {
    session_id: String,
    timeout_ms: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct VttyResizeArgs {
    session_id: String,
    cols: u64,
    rows: u64,
}

// ── Browser tools (HTTP proxy to shirabe / tairitsu daemon) ────────────

#[tool_router]
impl Server {
    #[tool(description = "Navigate to a URL")]
    async fn browser_navigate(
        &self,
        Parameters(args): Parameters<BrowserNavigateArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_daemon().await?;
        self.http_post_fire_and_forget("navigate", json!({"url": args.url}))
            .await?;
        Ok(Self::tool_result(format!("Navigated to {}", args.url)))
    }

    #[tool(description = "Go back to the previous page")]
    async fn browser_navigate_back(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_daemon().await?;
        self.http_post_fire_and_forget("back", json!({})).await?;
        Ok(Self::tool_result("Navigated back"))
    }

    #[tool(description = "Go forward to the next page")]
    async fn browser_navigate_forward(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_daemon().await?;
        self.http_post_fire_and_forget("forward", json!({})).await?;
        Ok(Self::tool_result("Navigated forward"))
    }

    #[tool(
        description = "Capture accessibility snapshot of the current page (DOM tree with roles, names, text). Better than screenshot for understanding page structure."
    )]
    async fn browser_snapshot(
        &self,
        Parameters(args): Parameters<SnapshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_daemon().await?;
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
        description = "Query a DOM element by CSS selector — returns its tag, text, html, attributes, visibility, bounding rect, and match count. Pass `attribute` to fetch just one attribute's value. Complements `browser_snapshot` (semantic a11y tree) with exact element details."
    )]
    async fn browser_dom(
        &self,
        Parameters(args): Parameters<DomQueryArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.ensure_daemon().await?;
        let mut query: Vec<(&str, &str)> = vec![("selector", args.selector.as_str())];
        if let Some(attr) = args.attribute.as_deref() {
            if !attr.is_empty() {
                query.push(("attribute", attr));
            }
        }
        if matches!(args.computed, Some(true)) {
            query.push(("computed", "true"));
        }
        if matches!(args.all, Some(true)) {
            query.push(("all", "true"));
        }
        let v = self.http_get("dom", &query).await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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
        self.ensure_daemon().await?;
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

    // ── VTty tools (delegated to the `kou` engine) ─────

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
        let env_pairs = parse_env_string(args.env.as_deref().unwrap_or(""));
        let env_refs: Vec<(&str, &str)> = env_pairs
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let info = self
            .vtty
            .launch(
                &args.command,
                resolved_cwd.as_deref(),
                &env_refs,
                args.cols.unwrap_or(120) as u16,
                args.rows.unwrap_or(40) as u16,
                args.name.as_deref(),
            )
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }

    #[tool(description = "Kill a virtual terminal session")]
    async fn vtty_kill(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self.vtty.kill(&args.session_id).await.ok_or_else(|| {
            McpError::internal_error(format!("Session '{}' not found", args.session_id), None)
        })?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }

    #[tool(
        description = "Send key sequences to a virtual terminal. Supports Enter, Tab, Escape, Backspace, Delete, Arrow keys, Home/End, PageUp/PageDown, F1-F12, Ctrl+X, Alt+X"
    )]
    async fn vtty_send_keys(
        &self,
        Parameters(args): Parameters<VttySendKeysArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.vtty
            .send_keys(&args.session_id, &args.keys)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "keys": args.keys, "sent": true}).to_string(),
        ))
    }

    #[tool(description = "Send text string to a virtual terminal")]
    async fn vtty_send_text(
        &self,
        Parameters(args): Parameters<VttySendTextArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.vtty
            .send_text(&args.session_id, &args.text)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "length": args.text.len(), "sent": true})
                .to_string(),
        ))
    }

    #[tool(
        description = "Capture current terminal screen content as text (text-only models) and/or as a rendered PNG image (vision-capable models). \
        The 'format' parameter controls output: 'text' (default) returns plain text, 'image' returns a rendered PNG, 'both' returns both. \
        The 'theme' parameter selects the PNG colour scheme (Windows Terminal schemes): campbell (default), campbell-powershell, vintage, one-half-dark, one-half-light, solarized-dark, solarized-light, tango-dark, tango-light, dimidium, ottosson, dark+, cga, ibm-5153, xterm. Unknown names fall back to campbell."
    )]
    async fn vtty_screenshot(
        &self,
        Parameters(args): Parameters<VttyScreenshotArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let fmt = args.format.as_deref().unwrap_or("text");
        let theme = kou::theme_by_name(args.theme.as_deref().unwrap_or("campbell"));

        let screen = self
            .vtty
            .screen(&args.session_id)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        let alive = self
            .vtty
            .ping(&args.session_id)
            .await
            .map(|i| i.alive)
            .unwrap_or(false);
        let text = screen.text();

        match fmt {
            "image" => {
                let b64 = self.render_png(&screen, theme)?;
                Ok(CallToolResult::success(vec![Content::image(
                    b64,
                    "image/png",
                )]))
            }
            "both" => {
                let b64 = self.render_png(&screen, theme)?;
                Ok(CallToolResult::success(vec![
                    Content::text(self.screen_text_json(&args.session_id, alive, &screen, &text)),
                    Content::image(b64, "image/png"),
                ]))
            }
            _ => Ok(Self::tool_result(self.screen_text_json(
                &args.session_id,
                alive,
                &screen,
                &text,
            ))),
        }
    }

    #[tool(description = "Wait for duration or until text appears on screen")]
    async fn vtty_wait(
        &self,
        Parameters(args): Parameters<VttyWaitArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let secs = args.seconds.unwrap_or(5.0);
        let pattern = args.pattern.unwrap_or_default();
        if !pattern.is_empty() {
            let deadline =
                std::time::Instant::now() + std::time::Duration::from_secs_f64(secs.min(1800.0));
            let mut found = false;
            while std::time::Instant::now() < deadline {
                let alive = self
                    .vtty
                    .ping(&args.session_id)
                    .await
                    .map(|i| i.alive)
                    .unwrap_or(false);
                if !alive {
                    break;
                }
                let hits = self
                    .vtty
                    .find_text(&args.session_id, &pattern)
                    .await
                    .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
                if !hits.is_empty() {
                    found = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            }
            let alive = self
                .vtty
                .ping(&args.session_id)
                .await
                .map(|i| i.alive)
                .unwrap_or(false);
            Ok(Self::tool_result(
                json!({"session_id": args.session_id, "pattern": pattern, "found": found, "alive": alive})
                    .to_string(),
            ))
        } else {
            let wait_secs = secs.min(1800.0) as u64;
            let mut alive = true;
            for _ in 0..(wait_secs * 20) {
                alive = self
                    .vtty
                    .ping(&args.session_id)
                    .await
                    .map(|i| i.alive)
                    .unwrap_or(false);
                if !alive {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Ok(Self::tool_result(
                json!({"session_id": args.session_id, "seconds_waited": secs, "alive": alive})
                    .to_string(),
            ))
        }
    }

    #[tool(
        description = "Wait until a VTty session has screen output (useful after vtty_launch for slow-starting commands). Returns immediately if output is already present."
    )]
    async fn vtty_ready(
        &self,
        Parameters(args): Parameters<VttyReadyArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let timeout_ms = args.timeout_ms.unwrap_or(30000);
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms);
        let mut ready = false;
        while std::time::Instant::now() < deadline {
            let has = self
                .vtty
                .has_output(&args.session_id)
                .await
                .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
            if has {
                ready = true;
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "ready": ready}).to_string(),
        ))
    }

    #[tool(
        description = "Get the scrollback buffer (history) of a virtual terminal session, including current screen content"
    )]
    async fn vtty_scrollback(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let text = self
            .vtty
            .scrollback(&args.session_id)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "text": text}).to_string(),
        ))
    }

    #[tool(description = "Resize a virtual terminal")]
    async fn vtty_resize(
        &self,
        Parameters(args): Parameters<VttyResizeArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let old = self
            .vtty
            .ping(&args.session_id)
            .await
            .map(|i| (i.cols, i.rows));
        self.vtty
            .resize(&args.session_id, args.cols as u16, args.rows as u16)
            .await
            .map_err(|e| McpError::internal_error(format!("{e}"), None))?;
        Ok(Self::tool_result(
            json!({"session_id": args.session_id, "old": old.map(|(c,r)| json!({"cols": c, "rows": r})), "new": {"cols": args.cols, "rows": args.rows}})
                .to_string(),
        ))
    }

    #[tool(description = "List all active virtual terminal sessions")]
    async fn vtty_list(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let sessions = self.vtty.list().await;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&sessions).unwrap_or_else(|_| "[]".to_string()),
        ))
    }

    #[tool(
        description = "Check if a VTty session's child process is still alive and refresh screen state"
    )]
    async fn vtty_ping(
        &self,
        Parameters(args): Parameters<VttySessionArgs>,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let info = self.vtty.ping(&args.session_id).await.ok_or_else(|| {
            McpError::internal_error(format!("Session '{}' not found", args.session_id), None)
        })?;
        Ok(Self::tool_result(
            serde_json::to_string_pretty(&info).unwrap_or_default(),
        ))
    }
}

// ── ServerHandler ────────────────────────────────────

#[tool_handler(router = Server::tool_router())]
impl ServerHandler for Server {}

// ── helpers ──────────────────────────────────────────

/// Parse an env-string of the form `"K=V\nK2=V2"` (newlines or commas) into
/// owned pairs. Malformed entries are dropped.
fn parse_env_string(raw: &str) -> Vec<(String, String)> {
    raw.split([',', '\n'])
        .filter_map(|pair| {
            let pair = pair.trim();
            if pair.is_empty() {
                return None;
            }
            let (k, v) = pair.split_once('=')?;
            Some((k.trim().to_string(), v.to_string()))
        })
        .collect()
}

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

// ── browser-server (shirabe / tairitsu daemon) resolution ─────────────

mod daemon {
    use std::path::PathBuf;

    use anyhow::{anyhow, Result};

    pub(super) async fn resolve_daemon_url() -> Result<String> {
        // An explicit shirabe debug-server URL wins (the browser backend was
        // extracted into the dedicated `shirabe` repo). Fall back to the
        // legacy tairitsu-daemon variable.
        for var in ["SHIRABE_URL", "TAIRITSU_DAEMON_URL"] {
            if let Ok(url) = std::env::var(var) {
                if !url.is_empty() {
                    return Ok(url);
                }
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
                let url = format!("http://localhost:{dp}");
                if check_daemon_health(&url).await {
                    return Ok(url);
                }
            }
            return Err(anyhow!(
                "Daemon found but debug API not responding. Start a shirabe debug server \
                 (shirabe serve) and point SHIRABE_URL at it."
            ));
        }

        let searched = search_project_roots_fallback();
        if let Some((_port, debug_port, _)) = try_read_ready_port_from_candidates(&searched) {
            if let Some(dp) = debug_port {
                let url = format!("http://localhost:{dp}");
                if check_daemon_health(&url).await {
                    return Ok(url);
                }
            }
            return Err(anyhow!(
                "Daemon found but debug API not responding. Start a shirabe debug server \
                 (shirabe serve) and point SHIRABE_URL at it."
            ));
        }
        Err(anyhow!(
            "No running browser debug server found. Start one with `shirabe serve` \
             (or set SHIRABE_URL / TAIRITSU_DAEMON_URL)."
        ))
    }

    async fn check_daemon_health(url: &str) -> bool {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
            .unwrap_or_default();
        client
            .get(format!("{url}/health"))
            .send()
            .await
            .is_ok_and(|resp| resp.status().is_success())
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
        let mut scan_dirs: Vec<PathBuf> = std::env::var("HOME")
            .ok()
            .map(PathBuf::from)
            .into_iter()
            .collect();
        if let Ok(dirs) = std::env::var("TAIRITSU_SCAN_DIRS") {
            scan_dirs.extend(dirs.split(':').map(PathBuf::from));
        }
        for scan_dir in scan_dirs {
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
                if trimmed.is_empty() || trimmed.starts_with("error:") {
                    let _ = std::fs::remove_file(&ready_path);
                    continue;
                }

                if let Ok(metadata) = std::fs::metadata(&ready_path) {
                    if let Ok(modified) = metadata.modified() {
                        if modified.elapsed().unwrap_or_default()
                            > std::time::Duration::from_secs(86400)
                        {
                            let pid_path = dir.join("tairitsu-packager.pid");
                            if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
                                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                                    if !is_process_running(pid) {
                                        let _ = std::fs::remove_file(&ready_path);
                                        let _ = std::fs::remove_file(&pid_path);
                                        continue;
                                    }
                                }
                            } else {
                                let _ = std::fs::remove_file(&ready_path);
                                continue;
                            }
                        }
                    }
                }

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

    #[cfg(unix)]
    fn is_process_running(pid: u32) -> bool {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    fn is_process_running(pid: u32) -> bool {
        std::process::Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

use daemon::resolve_daemon_url;

// ── public entry point ───────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct McpConfig {
    pub base_url: String,
}

pub async fn run(config: McpConfig) -> Result<()> {
    // Install the rustls crypto provider once, before any reqwest::Client is
    // built (font-fetch + browser HTTP proxy both use rustls-no-provider).
    let _ = rustls::crypto::ring::default_provider().install_default();

    let base_url = Arc::new(RwLock::new(String::new()));

    if !config.base_url.is_empty() {
        *base_url.write().await = config.base_url.clone();
    } else {
        let base_url_clone = base_url.clone();
        tokio::spawn(async move {
            loop {
                if let Ok(url) = resolve_daemon_url().await {
                    if !url.is_empty() {
                        *base_url_clone.write().await = url;
                        return;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        });
    }

    // Load VTty fonts once. Strategy: system fonts first (fast, zero-network,
    // includes CJK if NotoSansCJK is installed), then async fetch as fallback.
    // Fonts loaded at supersampled resolution (font_px × supersample).
    let font_px = FONT_PX * RENDER_SUPER as f32;
    let fonts = {
        let sys = kou::FontCache::from_system_fonts(font_px);
        if !sys.is_empty() {
            sys
        } else {
            let font_set = kou::FontSet::from_env();
            let remote = kou::FontCache::load_async(&font_set, font_px).await;
            if remote.is_empty() {
                kou::FontCache::empty()
            } else {
                remote
            }
        }
    };

    let server = Server {
        base_url: base_url.clone(),
        http: reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_default(),
        vtty: kou::VttyManager::new(),
        fonts: Arc::new(fonts),
    };

    let transport = rmcp::transport::stdio();
    let server_handle = server.serve(transport).await?;
    server_handle.waiting().await?;

    Ok(())
}
