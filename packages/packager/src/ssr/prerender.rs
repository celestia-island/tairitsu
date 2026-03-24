//! Static site generation utilities for SSR
//!
//! This module provides functions for pre-rendering routes to static HTML.

use crate::config::Config;
use std::path::PathBuf;

/// Configuration for static site generation
#[derive(Debug, Clone)]
pub struct PrerenderConfig {
    /// Routes to pre-render
    pub routes: Vec<String>,
    /// Output directory for generated HTML
    pub output_dir: PathBuf,
    /// Whether to include assets in the output
    pub include_assets: bool,
    /// Viewport width for SSR
    pub viewport_width: i32,
    /// Viewport height for SSR
    pub viewport_height: i32,
}

impl Default for PrerenderConfig {
    fn default() -> Self {
        Self {
            routes: vec!["".to_string()], // Root route
            output_dir: PathBuf::from("dist/prerendered"),
            include_assets: true,
            viewport_width: 1920,
            viewport_height: 1080,
        }
    }
}

impl PrerenderConfig {
    /// Create a new prerender config with custom routes
    pub fn new(routes: Vec<String>) -> Self {
        Self {
            routes,
            ..Default::default()
        }
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, output_dir: PathBuf) -> Self {
        self.output_dir = output_dir;
        self
    }

    /// Set viewport dimensions
    pub fn with_viewport(mut self, width: i32, height: i32) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    /// Enable or disable asset copying
    pub fn with_assets(mut self, include: bool) -> Self {
        self.include_assets = include;
        self
    }
}

/// Pre-render all configured routes to static HTML
pub fn prerender(config: &Config, prerender_config: &PrerenderConfig) -> crate::Result<()> {
    use std::fs;

    let output_dir = &prerender_config.output_dir;

    tracing::info!(
        "Pre-rendering {} routes to {}...",
        prerender_config.routes.len(),
        output_dir.display()
    );

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
        tracing::warn!("Failed to read index.html: {}", e);
        default_template(&config.package.name)
    });

    // Read WASM bytes
    let wasm_bytes = fs::read(&wasm_path)?;

    // Create output directory
    fs::create_dir_all(output_dir)?;

    #[cfg(feature = "ssr")]
    {
        use tairitsu_ssr::{render_full_page, SsrConfig};

        let ssr_config = SsrConfig::new(prerender_config.viewport_width, prerender_config.viewport_height);

        for route in &prerender_config.routes {
            tracing::info!("  Rendering /{}...", route);

            let html = render_full_page(&wasm_bytes, ssr_config.clone(), &template)
                .map_err(|e| crate::TairitsuPackagerError::BuildError(format!(
                    "Failed to render route '{}': {}",
                    route, e
                )))?;

            // Determine output path
            let output_path = if route.is_empty() {
                output_dir.join("index.html")
            } else {
                let route_path = output_dir.join(route).join("index.html");
                fs::create_dir_all(route_path.parent().unwrap())?;
                route_path
            };

            fs::write(&output_path, html)?;
            tracing::info!("    -> {}", output_path.display());
        }

        // Copy assets if requested
        if prerender_config.include_assets {
            tracing::info!("Copying assets...");
            copy_assets(dist_dir, output_dir)?;
        }
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (wasm_bytes, template);
        return Err(crate::TairitsuPackagerError::BuildError(
            "SSR feature is not enabled. Build with --features ssr".to_string(),
        ));
    }

    #[cfg(feature = "ssr")]
    {
        tracing::info!(
            "Pre-rendering complete! Output in: {}",
            output_dir.display()
        );

        Ok(())
    }
}

/// Copy static assets to the prerender output directory
fn copy_assets(dist_dir: &PathBuf, output_dir: &PathBuf) -> crate::Result<()> {
    use std::fs;
    use walkdir::WalkDir;

    let asset_extensions = [".wasm", ".js", ".css", ".png", ".jpg", ".jpeg", ".svg", ".ico", ".webp"];

    for entry in WalkDir::new(dist_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if asset_extensions.contains(&ext.to_string_lossy().as_ref()) {
                    let relative = path.strip_prefix(dist_dir).map_err(|e| {
                        crate::TairitsuPackagerError::BuildError(format!(
                            "Failed to get relative path: {}",
                            e
                        ))
                    })?;
                    let dest = output_dir.join(relative);

                    // Create parent directory if needed
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(path, &dest)?;
                    tracing::debug!("  Copied asset: {}", relative.display());
                }
            }
        }
    }

    Ok(())
}

/// Default HTML template when index.html is not found
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
