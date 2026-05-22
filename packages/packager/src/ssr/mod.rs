//! Server-Side Rendering support for Tairitsu
//!
//! This module provides SSR functionality including:
//! - Development server with server-side rendering
//! - Static site generation (pre-rendering)

pub mod prerender;

use anyhow::Result;
use std::path::PathBuf;

#[cfg(feature = "ssr")]
use tracing::error;
#[cfg(feature = "dev-server")]
use {
    axum::{
        extract::Request,
        middleware::{self, Next},
        response::{Html, Response},
        routing::get,
        Router,
    },
    tairitsu_ssr::{render_full_page, SsrConfig},
    tower_http::services::ServeDir,
};

use crate::config::Config;

/// Start the SSR development server
///
/// This server loads the WASM component and renders it on each request,
/// providing server-side rendered HTML to the client.
#[cfg(feature = "dev-server")]
pub async fn ssr_dev_server(
    config: &Config,
    port: u16,
    open: bool,
    _watch: bool,
) -> crate::Result<()> {
    use std::time::Instant;

    use crate::wasm::build_component;

    crate::log_info!("Tairitsu SSR development server");

    // Initial build
    let initial_started = Instant::now();
    build_component(config, false, None, false)?;
    let initial_elapsed = initial_started.elapsed();

    let dist_dir = config.build.output_dir.clone();

    // Load the WASM component for SSR
    let wasm_path = dist_dir.join(format!("{}.wasm", config.package.name));
    if !wasm_path.exists() {
        return Err(crate::TairitsuPackagerError::BuildError(format!(
            "WASM component not found at: {}",
            wasm_path.display()
        )));
    }

    // Read the template HTML
    let template_path = dist_dir.join("index.html");
    let template = std::fs::read_to_string(&template_path).unwrap_or_else(|e| {
        crate::log_warn!("Failed to read index.html: {}", e);
        default_template(&config.package.name)
    });

    // Read WASM bytes
    let wasm_bytes = std::fs::read(&wasm_path)?;
    let wasm_bytes_for_root = wasm_bytes.clone();
    let wasm_bytes_for_fallback = wasm_bytes.clone();
    let template_for_root = template.clone();
    let template_for_fallback = template.clone();
    let pkg_name_for_root = config.package.name.clone();
    let pkg_name_for_fallback = config.package.name.clone();

    // SSR handler for root route
    let ssr_handler = Router::new().route(
        "/",
        get(move || {
            let wasm_bytes = wasm_bytes_for_root.clone();
            let template = template_for_root.clone();
            let pkg = pkg_name_for_root.clone();
            async move {
                match render_ssr_page(&wasm_bytes, &template) {
                    Ok(html) => Html(html),
                    Err(e) => {
                        error!("SSR render error: {}", e);
                        Html(render_error_page(&pkg, &e.to_string()))
                    }
                }
            }
        }),
    );

    // SPA fallback for non-root paths
    let spa_fallback = Router::new().fallback(get(move || {
        let wasm_bytes = wasm_bytes_for_fallback.clone();
        let template = template_for_fallback.clone();
        let pkg = pkg_name_for_fallback.clone();
        async move {
            match render_ssr_page(&wasm_bytes, &template) {
                Ok(html) => Html(html),
                Err(e) => {
                    error!("SSR render error: {}", e);
                    Html(render_error_page(&pkg, &e.to_string()))
                }
            }
        }
    }));

    let app = Router::new()
        .merge(ssr_handler)
        .fallback_service(ServeDir::new(dist_dir).fallback(spa_fallback))
        .layer(middleware::from_fn(no_cache_headers));

    let (listener, actual_port) = bind_listener_with_fallback(port).await?;

    let last_build_line = format_last_build_line(true, initial_elapsed, None);

    let port_switched = if actual_port != port {
        Some(format!("Port switched: {} -> {}", port, actual_port))
    } else {
        None
    };

    print_status_panel(
        actual_port,
        &config.build.output_dir,
        Some(&last_build_line),
        port_switched.as_deref(),
    );

    if open || config.dev.open_browser {
        let url = format!("http://localhost:{}", actual_port);
        match webbrowser::open(&url) {
            Ok(_) => crate::log_ok!("Opening browser..."),
            Err(e) => {
                crate::log_warn!("Failed to open browser: {}", e);
            }
        }
    }

    crate::log_ok!("Server running at http://localhost:{}", actual_port);

    axum::serve(listener, app).await?;

    Ok(())
}

/// Render a page with SSR
#[cfg(feature = "dev-server")]
fn render_ssr_page(wasm_bytes: &[u8], template: &str) -> Result<String> {
    let ssr_config = SsrConfig::default();
    render_full_page(wasm_bytes, ssr_config, template)
}

/// Render an error page
#[cfg(feature = "dev-server")]
fn render_error_page(package_name: &str, error: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>SSR Error - {}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; }}
        .error {{ background: #fee; border: 1px solid #c00; padding: 1rem; border-radius: 4px; }}
        h1 {{ color: #c00; }}
    </style>
</head>
<body>
    <div class="error">
        <h1>Server-Side Rendering Error</h1>
        <p><strong>Package:</strong> {}</p>
        <p><strong>Error:</strong> {}</p>
    </div>
</body>
</html>"#,
        package_name, package_name, error
    )
}

/// Default HTML template when index.html is not found
#[cfg(feature = "dev-server")]
fn default_template(package_name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body>
    <div id="app"></div>
</body>
</html>"#,
        package_name
    )
}

/// Bind to the requested port, with fallback to next available port
#[cfg(feature = "dev-server")]
async fn bind_listener_with_fallback(port: u16) -> Result<(tokio::net::TcpListener, u16)> {
    use tokio::net::TcpListener;

    let try_port = port;
    match TcpListener::bind(("127.0.0.1", try_port)).await {
        Ok(listener) => Ok((listener, try_port)),
        Err(e) if try_port < 65535 => {
            crate::log_warn!("Port {} unavailable: {}", try_port, e);
            let listener = TcpListener::bind(("127.0.0.1", try_port + 1)).await?;
            Ok((listener, try_port + 1))
        }
        Err(e) => Err(e.into()),
    }
}

/// Middleware to add no-cache headers
#[cfg(feature = "dev-server")]
async fn no_cache_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "Cache-Control",
        "no-cache, no-store, must-revalidate".parse().unwrap(),
    );
    headers.insert("Pragma", "no-cache".parse().unwrap());
    headers.insert("Expires", "0".parse().unwrap());
    response
}

#[cfg(feature = "dev-server")]
fn format_last_build_line(
    success: bool,
    elapsed: std::time::Duration,
    error: Option<&str>,
) -> String {
    if success {
        format!("Built in {:.2}s", elapsed.as_secs_f64())
    } else {
        format!("Build failed: {}", error.unwrap_or("unknown error"))
    }
}

#[cfg(feature = "dev-server")]
fn print_status_panel(
    port: u16,
    dist_dir: &std::path::Path,
    build_line: Option<&str>,
    port_note: Option<&str>,
) {
    crate::log_ok!("Local:  http://localhost:{}", port);
    if let Some(note) = port_note {
        crate::log_info!("Note:   {}", note);
    }
    crate::log_info!("Output: {}", dist_dir.display());
    if let Some(line) = build_line {
        crate::log_info!("{}", line);
    }
}

/// Pre-render routes for static site generation
pub fn prerender_routes(
    config: &Config,
    routes: &[String],
    output_dir: &PathBuf,
) -> crate::Result<()> {
    #[cfg(feature = "ssr")]
    {
        use std::fs;

        use tairitsu_ssr::{render_full_page, SsrConfig};

        let discovered = config.discovered_routes();
        let effective_routes: Vec<String> = if routes.is_empty() && !discovered.is_empty() {
            discovered.iter().map(|r| r.path.clone()).collect()
        } else {
            routes.to_vec()
        };

        crate::log_info!("Pre-rendering {} routes...", effective_routes.len());

        // Build the component first
        crate::wasm::build_component(config, true, None, false)?;

        let dist_dir = &config.build.output_dir;
        let wasm_path = dist_dir.join(format!("{}.wasm", config.package.name));

        if !wasm_path.exists() {
            return Err(crate::TairitsuPackagerError::BuildError(format!(
                "WASM component not found at: {}",
                wasm_path.display()
            )));
        }

        // Read the template HTML
        let template_path = dist_dir.join("index.html");
        let template = fs::read_to_string(&template_path).unwrap_or_else(|e| {
            crate::log_warn!("Failed to read index.html: {}", e);
            default_template(&config.package.name)
        });

        // Read WASM bytes
        let wasm_bytes = fs::read(&wasm_path)?;

        // Create output directory
        fs::create_dir_all(output_dir)?;

        for route in &effective_routes {
            crate::log_info!("  Rendering /{}...", route);

            let clean_route = route.trim_start_matches('/');

            let ssr_config = SsrConfig::with_route(1920, 1080, route);

            let html = render_full_page(&wasm_bytes, ssr_config, &template).map_err(|e| {
                crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to render route '{}': {}",
                    route, e
                ))
            })?;

            let activated_html = activate_route_in_html(&html, route);

            // Determine output path
            let output_path = if clean_route.is_empty() {
                output_dir.join("index.html")
            } else {
                let route_path = output_dir.join(clean_route).join("index.html");
                fs::create_dir_all(route_path.parent().unwrap())?;
                route_path
            };

            fs::write(&output_path, activated_html)?;
            crate::log_info!("    -> {}", output_path.display());
        }

        crate::log_info!(
            "Pre-rendering complete! Output in: {}",
            output_dir.display()
        );
        Ok(())
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (config, routes, output_dir);
        Err(crate::TairitsuPackagerError::BuildError(
            "SSR feature is not enabled. Build with --features ssr".to_string(),
        ))
    }
}

/// Post-process rendered HTML to activate the correct SPA page.
///
/// Since tairitsu apps render all pages at once and use CSS show/hide,
/// this injects `is-active` class on the target page so search engines
/// and social media crawlers see the right content without JS execution.
fn activate_route_in_html(html: &str, route: &str) -> String {
    let clean_route = route.trim_matches('/');

    if clean_route.is_empty() {
        return html.to_string();
    }

    let page_id_map: std::collections::HashMap<String, &str> = [
        ("".to_string(), "home"),
        ("/".to_string(), "home"),
        ("/components".to_string(), "components"),
        ("/components/layer1/button".to_string(), "component-button"),
        ("/components/layer1/form".to_string(), "component-form"),
        (
            "/components/layer1/number-input".to_string(),
            "component-number-input",
        ),
        ("/components/layer1/search".to_string(), "component-search"),
        ("/components/layer1/switch".to_string(), "component-switch"),
        (
            "/components/layer1/feedback".to_string(),
            "component-feedback",
        ),
        (
            "/components/layer1/display".to_string(),
            "component-display",
        ),
        ("/components/layer1/avatar".to_string(), "component-avatar"),
        ("/components/layer1/image".to_string(), "component-image"),
        ("/components/layer1/tag".to_string(), "component-tag"),
        ("/components/layer1/empty".to_string(), "component-empty"),
        (
            "/components/layer1/comment".to_string(),
            "component-comment",
        ),
        (
            "/components/layer1/description-list".to_string(),
            "component-description-list",
        ),
        (
            "/components/layer2/navigation".to_string(),
            "component-navigation",
        ),
        ("/components/layer2/data".to_string(), "component-data"),
        ("/components/layer2/table".to_string(), "component-table"),
        ("/components/layer2/tree".to_string(), "component-tree"),
        (
            "/components/layer2/pagination".to_string(),
            "component-pagination",
        ),
        ("/components/layer2/qrcode".to_string(), "component-qrcode"),
        (
            "/components/layer2/timeline".to_string(),
            "component-timeline",
        ),
        (
            "/components/layer2/form".to_string(),
            "component-form-composed",
        ),
        (
            "/components/layer2/cascader".to_string(),
            "component-cascader",
        ),
        (
            "/components/layer2/transfer".to_string(),
            "component-transfer",
        ),
        (
            "/components/layer2/collapsible".to_string(),
            "component-collapsible",
        ),
        (
            "/components/layer2/feedback".to_string(),
            "component-feedback-composed",
        ),
        ("/components/layer3/media".to_string(), "component-media"),
        ("/components/layer3/editor".to_string(), "component-editor"),
        (
            "/components/layer3/visualization".to_string(),
            "component-visualization",
        ),
        (
            "/components/layer3/user-guide".to_string(),
            "component-user-guide",
        ),
        (
            "/components/layer3/zoom-controls".to_string(),
            "component-zoom-controls",
        ),
        ("/system/architecture".to_string(), "system-architecture"),
        ("/system/design-tokens".to_string(), "system-design-tokens"),
        (
            "/guides/getting-started".to_string(),
            "guide-getting-started",
        ),
        ("/guides/theminging".to_string(), "guide-theming"),
        ("/demos/showcase".to_string(), "demos-showcase"),
        ("/demos/form".to_string(), "demos-form"),
        ("/demos/dashboard".to_string(), "demos-dashboard"),
        ("/demos/video".to_string(), "demos-video"),
    ]
    .iter()
    .cloned()
    .collect();

    match page_id_map
        .get(&format!("/{}", clean_route))
        .or(page_id_map.get(route))
    {
        Some(page_id) => {
            let target_id = format!("page-{}", page_id);
            html.replace(
                &format!(r#"id="{}""#, target_id),
                &format!(r#"id="{}" class="ts-page is-active""#, target_id),
            )
        }
        None => html.to_string(),
    }
}

/// Default HTML template when index.html is not found
#[cfg(not(feature = "dev-server"))]
fn default_template(package_name: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
</head>
<body>
    <div id="app"></div>
</body>
</html>"#,
        package_name
    )
}
