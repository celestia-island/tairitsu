use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, ListenerOptions, ToFsName,
};
use tairitsu_shared::*;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

const PLUGIN_NAME: &str = "debug-browser";
const PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");
const PLUGIN_DESCRIPTION: &str = "Headless Chromium CDP automation via debug HTTP API";
const DEFAULT_VIEWPORT_W: u32 = 1280;
const DEFAULT_VIEWPORT_H: u32 = 720;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("tairitsu_mcp_plugin_browser=info".parse()?),
        )
        .init();

    let socket_path = parse_args()?;

    let _ = std::fs::remove_file(&socket_path);

    let listener = ListenerOptions::new()
        .name(socket_path.clone().to_fs_name::<GenericFilePath>()?)
        .create_tokio()
        .map_err(|e| anyhow::anyhow!("bind socket {}: {}", socket_path.display(), e))?;

    tracing::info!("[{}] Listening on {}", PLUGIN_NAME, socket_path.display());

    let (browser, page) = launch_browser().await?;

    tracing::info!("[{}] Browser launched, waiting for host", PLUGIN_NAME);

    let stream: Stream = listener
        .accept()
        .await
        .map_err(|e| anyhow::anyhow!("accept connection: {}", e))?;

    tracing::info!("[{}] Host connected", PLUGIN_NAME);

    run_event_loop(stream, &browser, &page).await?;

    let _ = std::fs::remove_file(&socket_path);
    Ok(())
}

fn parse_args() -> anyhow::Result<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--socket" {
        anyhow::bail!(
            "Usage: {} --socket <socket-path>",
            args.get(0).map(|s| s.as_str()).unwrap_or("plugin")
        );
    }
    Ok(PathBuf::from(&args[2]))
}

async fn launch_browser() -> anyhow::Result<(Browser, chromiumoxide::Page)> {
    let config = resolve_browser_config()?;

    let _ = std::fs::remove_file("/tmp/chromiumoxide-runner/SingletonLock");

    let (browser, mut handler) = Browser::launch(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to launch Chrome: {e}"))?;

    tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if event.is_err() {
                break;
            }
        }
    });

    let page = browser
        .new_page("about:blank")
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create page: {e}"))?;

    tracing::info!("[{}] Chrome started successfully", PLUGIN_NAME);
    Ok((browser, page))
}

fn resolve_browser_config() -> anyhow::Result<BrowserConfig> {
    let mut builder = BrowserConfig::builder()
        .window_size(DEFAULT_VIEWPORT_W, DEFAULT_VIEWPORT_H)
        .arg("--no-sandbox")
        .arg("--disable-dev-shm-usage")
        .arg("--disable-gpu");

    if let Ok(exe) = std::env::var("CHROME_PATH") {
        if !exe.is_empty() {
            tracing::info!("[{}] Using CHROME_PATH={}", PLUGIN_NAME, exe);
            builder = builder.chrome_executable(exe);
            return Ok(builder.build().map_err(|e| anyhow::anyhow!("Bad browser config: {e}"))?);
        }
    }

    if let Ok(exe) = which_chromium() {
        tracing::info!("[{}] Found browser: {}", PLUGIN_NAME, exe);
        builder = builder.chrome_executable(exe);
        return Ok(builder.build().map_err(|e| anyhow::anyhow!("Bad browser config: {e}"))?);
    }

    tracing::info!("[{}] No browser found, auto-downloading Chromium...", PLUGIN_NAME);
    let rt = tokio::runtime::Handle::current();
    let fetcher = chromiumoxide::fetcher::BrowserFetcher::new(
        chromiumoxide::fetcher::BrowserFetcherOptions::builder()
            .build()
            .map_err(|e| anyhow::anyhow!("Fetcher config: {e}"))?,
    );
    let info = rt.block_on(fetcher.fetch())
        .map_err(|e| anyhow::anyhow!("Fetcher download: {e}"))?;
    tracing::info!("[{}] Chromium downloaded: {}", PLUGIN_NAME, info.executable_path.display());
    builder = builder.chrome_executable(&info.executable_path);
    Ok(builder.build().map_err(|e| anyhow::anyhow!("Bad browser config: {e}"))?)
}

fn which_chromium() -> Result<String, ()> {
    let candidates = [
        "chromium-browser",
        "chromium",
        "google-chrome",
        "google-chrome-stable",
        "chrome",
    ];
    for name in &candidates {
        if let Ok(output) = std::process::Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }
    }
    Err(())
}

async fn run_event_loop(
    stream: Stream,
    _browser: &Browser,
    page: &chromiumoxide::Page,
) -> anyhow::Result<()> {
    let mut stream = BufReader::new(stream);

    let handshake = Message::Handshake(Handshake {
        protocol_version: PROTOCOL_VERSION,
        name: PLUGIN_NAME.to_string(),
        version: PLUGIN_VERSION.to_string(),
        capabilities: vec![caps::DEBUG_BROWSER.to_string()],
        description: Some(PLUGIN_DESCRIPTION.to_string()),
    });

    send_message(&mut stream, &handshake).await?;
    tracing::info!("[{}] Handshake sent", PLUGIN_NAME);

    let mut line = String::new();
    if stream.read_line(&mut line).await? > 0 {
        if let Ok(msg) = serde_json::from_str::<Message>(line.trim()) {
            if let Message::HandshakeAck(ack) = msg {
                if !ack.accepted {
                    tracing::error!("[{}] Handshake rejected: {:?}", PLUGIN_NAME, ack.reason);
                    return Ok(());
                }
                tracing::info!("[{}] Handshake accepted", PLUGIN_NAME);
            }
        }
    }

    let page = Arc::new(page.clone());

    loop {
        line.clear();
        let bytes_read = stream.read_line(&mut line).await?;
        if bytes_read == 0 {
            tracing::info!("[{}] Host disconnected", PLUGIN_NAME);
            break;
        }

        let msg: Message = match serde_json::from_str(line.trim()) {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("[{}] Invalid message: {} ({})", PLUGIN_NAME, e, line.trim());
                continue;
            }
        };

        match msg {
            Message::Request(req) => {
                let p = page.clone();
                let resp = handle_request(&req, &p).await;
                send_message(&mut stream, &resp).await?;
            }
            _ => {
                tracing::warn!("[{}] Unexpected message type: {:?}", PLUGIN_NAME, msg);
            }
        }
    }

    Ok(())
}

async fn handle_request(req: &Request, page: &Arc<chromiumoxide::Page>) -> Message {
    match req.method.as_str() {
        "ping" => Message::Response(Response {
            id: req.id,
            result: Some(serde_json::json!({ "pong": true })),
        }),

        "browser.navigate" => {
            let url = req
                .params
                .as_ref()
                .and_then(|p| p.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("about:blank");
            let wait_for = req
                .params
                .as_ref()
                .and_then(|p| p.get("wait_for"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            match cmd_navigate(page, url, wait_for.as_deref()).await {
                Ok(resp) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::to_value(resp).unwrap_or_default()),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.screenshot" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let full_page = req
                .params
                .as_ref()
                .and_then(|p| p.get("full_page"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            match cmd_screenshot(page, selector.as_deref(), full_page).await {
                Ok(resp) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::to_value(resp).unwrap_or_default()),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.click" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match cmd_click(page, selector).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "clicked": selector })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.type" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let text = req
                .params
                .as_ref()
                .and_then(|p| p.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let clear_first = req
                .params
                .as_ref()
                .and_then(|p| p.get("clear_first"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let submit = req
                .params
                .as_ref()
                .and_then(|p| p.get("submit"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            match cmd_type(page, selector, text, clear_first, submit).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "typed": text.len() })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.press" => {
            let key = req
                .params
                .as_ref()
                .and_then(|p| p.get("key"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match cmd_press(page, key).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "pressed": key })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.scroll" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let x = req
                .params
                .as_ref()
                .and_then(|p| p.get("x"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let y = req
                .params
                .as_ref()
                .and_then(|p| p.get("y"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            match cmd_scroll(page, selector.as_deref(), x, y).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "scrolled": true })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.evaluate" => {
            let expression = req
                .params
                .as_ref()
                .and_then(|p| p.get("expression"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            match cmd_evaluate(page, expression).await {
                Ok(val) => Message::Response(Response {
                    id: req.id,
                    result: Some(val),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.a11y" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let depth = req
                .params
                .as_ref()
                .and_then(|p| p.get("depth"))
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as u32;

            match cmd_a11y(page, selector.as_deref(), depth).await {
                Ok(nodes) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "nodes": nodes })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.dom" => {
            let selector = req
                .params
                .as_ref()
                .and_then(|p| p.get("selector"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let attribute = req
                .params
                .as_ref()
                .and_then(|p| p.get("attribute"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            match cmd_dom_query(page, selector, attribute.as_deref()).await {
                Ok(val) => Message::Response(Response {
                    id: req.id,
                    result: Some(val),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.resize" => {
            let width = req
                .params
                .as_ref()
                .and_then(|p| p.get("width"))
                .and_then(|v| v.as_u64())
                .unwrap_or(DEFAULT_VIEWPORT_W as u64) as u32;
            let height = req
                .params
                .as_ref()
                .and_then(|p| p.get("height"))
                .and_then(|v| v.as_u64())
                .unwrap_or(DEFAULT_VIEWPORT_H as u64) as u32;

            match cmd_resize(page, width, height).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "width": width, "height": height })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        "browser.viewport" => match cmd_viewport(page).await {
            Ok(val) => Message::Response(Response {
                id: req.id,
                result: Some(val),
            }),
            Err(e) => error_response(req.id, e),
        },

        "browser.network" => match cmd_network(page).await {
            Ok(val) => Message::Response(Response {
                id: req.id,
                result: Some(val),
            }),
            Err(e) => error_response(req.id, e),
        },

        "browser.performance" => match cmd_performance(page).await {
            Ok(val) => Message::Response(Response {
                id: req.id,
                result: Some(val),
            }),
            Err(e) => error_response(req.id, e),
        },

        "browser.drag" => {
            let from = req
                .params
                .as_ref()
                .and_then(|p| p.get("from_selector"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let to = req
                .params
                .as_ref()
                .and_then(|p| p.get("to_selector"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let steps = req
                .params
                .as_ref()
                .and_then(|p| p.get("steps"))
                .and_then(|v| v.as_u64())
                .unwrap_or(10) as u32;

            match cmd_drag(page, from, to, steps).await {
                Ok(()) => Message::Response(Response {
                    id: req.id,
                    result: Some(serde_json::json!({ "dragged": true })),
                }),
                Err(e) => error_response(req.id, e),
            }
        }

        other => {
            tracing::warn!("[{}] Unknown method: {}", PLUGIN_NAME, other);
            Message::Error(ErrorResponse {
                id: req.id,
                error: ErrorBody {
                    code: -1,
                    message: format!("Unknown method: {}", other),
                },
            })
        }
    }
}

fn error_response(id: u64, msg: String) -> Message {
    Message::Error(ErrorResponse {
        id,
        error: ErrorBody {
            code: -1,
            message: msg,
        },
    })
}

async fn send_message(
    stream: &mut BufReader<Stream>,
    msg: &Message,
) -> anyhow::Result<()> {
    let json = serde_json::to_string(msg)?;
    stream.get_mut().write_all(json.as_bytes()).await?;
    stream.get_mut().write_all(b"\n").await?;
    stream.get_mut().flush().await?;
    Ok(())
}

// ── Browser command implementations ──────────────────────────────────────────

#[derive(serde::Serialize)]
struct NavigateResult {
    url: String,
    title: String,
}

#[derive(serde::Serialize)]
struct ScreenshotResult {
    data: String,
    mime_type: String,
    width: u32,
    height: u32,
}

async fn cmd_navigate(
    page: &chromiumoxide::Page,
    url: &str,
    wait_for: Option<&str>,
) -> Result<NavigateResult, String> {
    page.goto(url).await.map_err(|e| format!("navigate: {e}"))?;
    if matches!(wait_for, Some("hydration") | Some("ready")) {
        tokio::time::sleep(Duration::from_secs(3)).await;
    } else if matches!(wait_for, Some("load")) {
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    let title = page.get_title().await.ok().flatten().unwrap_or_default();
    Ok(NavigateResult {
        url: url.to_string(),
        title,
    })
}

async fn cmd_screenshot(
    page: &chromiumoxide::Page,
    selector: Option<&str>,
    full_page: bool,
) -> Result<ScreenshotResult, String> {
    if let Some(sel) = selector {
        let element = page
            .find_element(sel)
            .await
            .map_err(|e| format!("element not found: {e}"))?;
        let png = element
            .screenshot(CaptureScreenshotFormat::Png)
            .await
            .map_err(|e| format!("screenshot element: {e}"))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
        return Ok(ScreenshotResult {
            data: b64,
            mime_type: "image/png".into(),
            width: 0,
            height: 0,
        });
    }
    let mut params = ScreenshotParams::builder()
        .format(CaptureScreenshotFormat::Png)
        .omit_background(false);
    if full_page {
        params = params.full_page(true);
    }
    let png = page
        .screenshot(params.build())
        .await
        .map_err(|e| format!("screenshot: {e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    Ok(ScreenshotResult {
        data: b64,
        mime_type: "image/png".into(),
        width: DEFAULT_VIEWPORT_W,
        height: DEFAULT_VIEWPORT_H,
    })
}

async fn cmd_click(page: &chromiumoxide::Page, selector: &str) -> Result<(), String> {
    page.find_element(selector)
        .await
        .map_err(|e| format!("click: element not found: {e}"))?
        .click()
        .await
        .map_err(|e| format!("click: {e}"))?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn cmd_type(
    page: &chromiumoxide::Page,
    selector: &str,
    text: &str,
    clear_first: bool,
    _submit: bool,
) -> Result<(), String> {
    let el = page
        .find_element(selector)
        .await
        .map_err(|e| format!("type: element not found: {e}"))?;
    el.click().await.map_err(|e| format!("type click: {e}"))?;
    if clear_first {
        let js = format!(
            r#"(() => {{ const el = document.querySelector({selector:?}); if (el) {{ el.value = ''; el.dispatchEvent(new Event('input', {{bubbles: true}})); }} }})()"#,
        );
        page.evaluate(js)
            .await
            .map_err(|e| format!("type clear: {e}"))?;
    }
    el.type_str(text)
        .await
        .map_err(|e| format!("type: {e}"))?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn cmd_press(page: &chromiumoxide::Page, key: &str) -> Result<(), String> {
    let js = format!(
        r#"(() => {{ document.dispatchEvent(new KeyboardEvent('keydown', {{key: {key:?}, code: {key:?}, bubbles: true}})); document.dispatchEvent(new KeyboardEvent('keyup', {{key: {key:?}, code: {key:?}, bubbles: true}})); }})()"#,
    );
    page.evaluate(js)
        .await
        .map_err(|e| format!("press: {e}"))?;
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(())
}

async fn cmd_scroll(
    page: &chromiumoxide::Page,
    selector: Option<&str>,
    x: f64,
    y: f64,
) -> Result<(), String> {
    let js = if let Some(sel) = selector {
        format!(
            r#"(() => {{ const el = document.querySelector({sel:?}); if (el) el.scrollBy({x}, {y}); }})()"#,
        )
    } else {
        format!(r#"window.scrollBy({x}, {y})"#)
    };
    page.evaluate(js)
        .await
        .map_err(|e| format!("scroll: {e}"))?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    Ok(())
}

async fn cmd_evaluate(
    page: &chromiumoxide::Page,
    expression: &str,
) -> Result<serde_json::Value, String> {
    let result = page
        .evaluate(expression.to_string())
        .await
        .map_err(|e| format!("evaluate: {e}"))?;
    Ok(result.into_value().unwrap_or(serde_json::Value::Null))
}

async fn cmd_a11y(
    page: &chromiumoxide::Page,
    selector: Option<&str>,
    depth: u32,
) -> Result<serde_json::Value, String> {
    let sel_js = match selector {
        Some(s) => format!("document.querySelector({s:?})"),
        None => "document.body".to_string(),
    };
    let js_body = r#"
(function(){
function getA11y(el,d,maxD){
if(!el||d>maxD)return null;
var tagRoles={BUTTON:'button',SELECT:'listbox',OPTION:'option',A:'link',H1:'heading',H2:'heading',H3:'heading',H4:'heading',H5:'heading',H6:'heading',NAV:'navigation',MAIN:'main',HEADER:'banner',FOOTER:'contentinfo',ASIDE:'complementary',FORM:'form',TABLE:'table',UL:'list',OL:'list',LI:'listitem',IMG:'img',SVG:'img',PROGRESS:'progressbar',METER:'meter',DIALOG:'dialog',DETAILS:'group',SUMMARY:'button',FIELDSET:'group'};
var inputRoles={checkbox:'checkbox',radio:'radio'};
var role=el.getAttribute('role')||(el.tagName?(tagRoles[el.tagName]||(el.tagName==='INPUT'?(inputRoles[el.getAttribute('type')]||'textbox'):(el.tagName==='TEXTAREA'?'textbox':undefined))):undefined);
var name=el.getAttribute('aria-label')||el.getAttribute('title')||((el.tagName==='INPUT'||el.tagName==='TEXTAREA')?el.getAttribute('placeholder'):null)||(el.tagName==='IMG'?el.getAttribute('alt'):null)||null;
var desc=el.getAttribute('aria-description')||null;
var states=[];
if(el.disabled)states.push('disabled');
if(el.getAttribute('aria-hidden')==='true')states.push('hidden');
if(el.getAttribute('aria-expanded')==='true')states.push('expanded');
if(el.getAttribute('aria-expanded')==='false')states.push('collapsed');
if(el.getAttribute('aria-selected')==='true')states.push('selected');
if(el.getAttribute('aria-checked')==='true')states.push('checked');
if(el.getAttribute('aria-checked')==='mixed')states.push('mixed');
var children=[];
if(d<maxD){for(var i=0;i<el.children.length;i++){var child=getA11y(el.children[i],d+1,maxD);if(child)children.push(child)}}
return{name:name,role:role||null,description:desc,states:states,tag:el.tagName?el.tagName.toLowerCase():null,children:children}
}
var root=SEL_JS;
if(!root)throw'element not found';
var tree=getA11y(root,0,DEPTH);
return JSON.stringify([tree])
})()
"#
    .replace("SEL_JS", &sel_js)
        .replace("DEPTH", &depth.to_string());

    let val = page
        .evaluate(js_body)
        .await
        .map_err(|e| format!("a11y: {e}"))?;
    let json_str: String = val.into_value().map_err(|e| format!("a11y parse: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| format!("a11y deserialize: {e}"))
}

async fn cmd_dom_query(
    page: &chromiumoxide::Page,
    selector: &str,
    attribute: Option<&str>,
) -> Result<serde_json::Value, String> {
    if let Some(attr) = attribute {
        let js = format!(
            "(() => {{ const el = document.querySelector({sel:?}); if (!el) return null; return el.getAttribute({attr:?}); }})()",
            sel = selector,
            attr = attr,
        );
        let val = page
            .evaluate(js)
            .await
            .map_err(|e| format!("dom query: {e}"))?;
        let r: Option<String> = val.into_value().ok();
        let count = if r.is_some() { 1 } else { 0 };
        return Ok(serde_json::json!({
            "text": r,
            "count": count,
        }));
    }
    let js = format!(
        r#"(() => {{ const els = document.querySelectorAll({sel:?}); if (!els.length) throw 'not found'; const el = els[0]; const r = el.getBoundingClientRect(); return JSON.stringify({{ tag: el.tagName.toLowerCase(), text: (el.textContent || '').trim().substring(0, 2000), html: el.outerHTML.substring(0, 5000), attrs: Object.fromEntries(Array.from(el.attributes).map(a => [a.name, a.value])), visible: r.width > 0 && r.height > 0, count: els.length, rect: {{ x: r.x, y: r.y, width: r.width, height: r.height }} }}); }})()"#,
        sel = selector,
    );
    let val = page
        .evaluate(js)
        .await
        .map_err(|e| format!("dom query: {e}"))?;
    let json_str: String = val
        .into_value()
        .map_err(|e| format!("dom query parse: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| format!("dom query deserialize: {e}"))
}

async fn cmd_resize(
    page: &chromiumoxide::Page,
    width: u32,
    height: u32,
) -> Result<(), String> {
    let params = SetDeviceMetricsOverrideParams::builder()
        .width(width as i64)
        .height(height as i64)
        .device_scale_factor(1.0)
        .mobile(false)
        .build()
        .map_err(|e| format!("resize build: {e}"))?;
    page.execute(params)
        .await
        .map_err(|e| format!("resize: {e}"))?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(())
}

async fn cmd_viewport(page: &chromiumoxide::Page) -> Result<serde_json::Value, String> {
    let js = r#"(() => { const dpr = window.devicePixelRatio || 1; return JSON.stringify({ width: window.innerWidth, height: window.innerHeight, device_pixel_ratio: dpr }); })()"#;
    let val = page
        .evaluate(js)
        .await
        .map_err(|e| format!("viewport: {e}"))?;
    let json_str: String = val
        .into_value()
        .map_err(|e| format!("viewport parse: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| format!("viewport deserialize: {e}"))
}

async fn cmd_network(page: &chromiumoxide::Page) -> Result<serde_json::Value, String> {
    let js = r#"(() => { var entries = performance.getEntriesByType('resource').slice(0, 100).map(function(e) { return { name: e.name, type: e.initiatorType || 'unknown', duration: Math.round(e.duration * 100) / 100, size: e.transferSize || 0, url: e.name }; }); return JSON.stringify({ resources: entries }); })()"#;
    let val = page
        .evaluate(js)
        .await
        .map_err(|e| format!("network: {e}"))?;
    let json_str: String = val
        .into_value()
        .map_err(|e| format!("network parse: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| format!("network deserialize: {e}"))
}

async fn cmd_performance(page: &chromiumoxide::Page) -> Result<serde_json::Value, String> {
    let js = r#"(() => { var nav = performance.getEntriesByType('navigation')[0] || {}; var fcp = null; try { fcp = performance.getEntriesByName('first-contentful-paint')[0].startTime || null; } catch(e) {} var dn = document.querySelectorAll('*').length; var heap = null; try { heap = Math.round((performance.memory ? performance.memory.usedJSHeapSize : 0) / 1048576 * 100) / 100; } catch(e) {} return JSON.stringify({ dom_content_loaded_ms: Math.round((nav.domContentLoadedEventEnd - nav.startTime) * 100) / 100 || null, dom_complete_ms: Math.round((nav.domComplete - nav.startTime) * 100) / 100 || null, load_event_ms: Math.round((nav.loadEventEnd - nav.startTime) * 100) / 100 || null, fcp_ms: fcp ? Math.round(fcp * 100) / 100 : null, dom_nodes: dn, js_heap_used_mb: heap, timestamp: new Date().toISOString() }); })()"#;
    let val = page
        .evaluate(js)
        .await
        .map_err(|e| format!("performance: {e}"))?;
    let json_str: String = val
        .into_value()
        .map_err(|e| format!("performance parse: {e}"))?;
    serde_json::from_str::<serde_json::Value>(&json_str).map_err(|e| format!("performance deserialize: {e}"))
}

async fn cmd_drag(
    page: &chromiumoxide::Page,
    from_selector: &str,
    to_selector: &str,
    steps: u32,
) -> Result<(), String> {
    let js = format!(
        r#"(() => {{ var src = document.querySelector({from:?}); var dst = document.querySelector({to:?}); if (!src || !dst) throw 'element not found'; var sr = src.getBoundingClientRect(); var dr = dst.getBoundingClientRect(); var sx = sr.x + sr.width/2, sy = sr.y + sr.height/2; var dx = dr.x + dr.width/2, dy = dr.y + dr.height/2; src.dispatchEvent(new MouseEvent('mousedown', {{clientX: sx, clientY: sy, bubbles: true}})); for (var i = 1; i <= {steps}; i++) {{ var t = i/{steps}; var cx = sx + (dx - sx)*t, cy = sy + (dy - sy)*t; document.dispatchEvent(new MouseEvent('mousemove', {{clientX: cx, clientY: cy, bubbles: true}})); }} dst.dispatchEvent(new MouseEvent('mouseup', {{clientX: dx, clientY: dy, bubbles: true}})); dst.dispatchEvent(new MouseEvent('drop', {{clientX: dx, clientY: dy, bubbles: true}})); }})()"#,
        from = from_selector,
        to = to_selector,
        steps = steps,
    );
    page.evaluate(js)
        .await
        .map_err(|e| format!("drag: {e}"))?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    Ok(())
}
