//! Server-Side Rendering support for Tairitsu
//!
//! This module provides SSR functionality including:
//! - Development server with server-side rendering
//! - Static site generation (pre-rendering)

use crate::config::Config;
use anyhow::Result;
use std::path::PathBuf;
use tracing::{error, warn};

#[cfg(feature = "ssr")]
use tracing::info;

#[cfg(feature = "dev-server")]
use {
    axum::{
        Router,
        extract::Request,
        middleware::{self, Next},
        response::{Html, Response},
        routing::get,
    },
    tairitsu_ssr::{SsrConfig, render_full_page},
    tower_http::services::ServeDir,
};

pub mod prerender;

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
    use crate::wasm::build_component;
    use std::time::Instant;
    use tracing::info;

    let divider = panel_divider();
    println!("{}", divider);
    println!("  Tairitsu  SSR  Development Server");
    println!("{}", divider);
    println!();

    // Initial build
    let initial_started = Instant::now();
    build_component(config, false, None)?;
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
        warn!("Failed to read index.html: {}", e);
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
            Ok(_) => println!("  Opening browser..."),
            Err(e) => warn!("Failed to open browser: {}", e),
        }
    }

    println!();
    info!("Server running at http://localhost:{}", actual_port);
    println!();

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
            warn!("Port {} unavailable: {}", try_port, e);
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

/// Print the divider panel
#[cfg(feature = "dev-server")]
fn panel_divider() -> String {
    "─".repeat(40)
}

/// Format the last build line
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

/// Print the status panel
#[cfg(feature = "dev-server")]
fn print_status_panel(
    port: u16,
    dist_dir: &std::path::Path,
    build_line: Option<&str>,
    port_note: Option<&str>,
) {
    println!("  Server:");
    println!("    Local:  http://localhost:{}", port);
    if let Some(note) = port_note {
        println!("    Note:   {}", note);
    }
    println!();
    println!("  Output:");
    println!("    {}", dist_dir.display());
    if let Some(line) = build_line {
        println!();
        println!("  {}", line);
    }
    println!();
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
        use tairitsu_ssr::{SsrConfig, render_full_page};

        info!("Pre-rendering {} routes...", routes.len());

        // Build the component first
        crate::wasm::build_component(config, true, None)?;

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
            warn!("Failed to read index.html: {}", e);
            default_template(&config.package.name)
        });

        // Read WASM bytes
        let wasm_bytes = fs::read(&wasm_path)?;

        // Create output directory
        fs::create_dir_all(output_dir)?;

        let ssr_config = SsrConfig::default();

        for route in routes {
            info!("  Rendering /{}...", route);

            let html =
                render_full_page(&wasm_bytes, ssr_config.clone(), &template).map_err(|e| {
                    crate::TairitsuPackagerError::BuildError(format!(
                        "Failed to render route '{}': {}",
                        route, e
                    ))
                })?;

            // Determine output path
            let output_path = if route.is_empty() {
                output_dir.join("index.html")
            } else {
                let route_path = output_dir.join(route).join("index.html");
                fs::create_dir_all(route_path.parent().unwrap())?;
                route_path
            };

            fs::write(&output_path, html)?;
            info!("    -> {}", output_path.display());
        }

        info!(
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
