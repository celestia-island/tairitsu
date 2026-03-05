use indicatif::{ProgressBar, ProgressStyle};
use crate::config::Config;

pub fn build(config: &Config, release: bool) -> crate::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
    );

    // Step 1: Check wasm32-unknown-unknown target
    pb.set_message("Checking WASM target...");
    check_wasm_target()?;

    // Step 2: Build WASM
    pb.set_message("Compiling WASM...");
    build_wasm(release)?;

    // Step 3: Run wasm-bindgen
    pb.set_message("Generating JS bindings...");
    run_wasm_bindgen(config)?;

    // Step 4: Generate HTML
    pb.set_message("Generating HTML...");
    generate_html(config)?;

    pb.finish_with_message("Build complete! ✅");

    println!("\nOutput: {}", config.build.output_dir.display());
    println!("Run `tairitsu preview` to see the result");

    Ok(())
}

fn check_wasm_target() -> crate::Result<()> {
    let output = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;

    let targets = String::from_utf8_lossy(&output.stdout);
    if !targets.contains("wasm32-unknown-unknown") {
        return Err(crate::TairitsuPackagerError::BuildError(
            "wasm32-unknown-unknown target not installed. Run: rustup target add wasm32-unknown-unknown".to_string()
        ));
    }

    Ok(())
}

fn build_wasm(release: bool) -> crate::Result<()> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.args(["build", "--target", "wasm32-unknown-unknown"]);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(crate::TairitsuPackagerError::BuildError(
            "Cargo build failed".to_string()
        ));
    }

    Ok(())
}

fn run_wasm_bindgen(config: &Config) -> crate::Result<()> {
    let pkg_name = &config.package.name;
    let profile = if config.build.optimize { "release" } else { "debug" };
    let wasm_path = format!("target/wasm32-unknown-unknown/{}/{}.wasm", profile, pkg_name.replace('-', "_"));

    // Create output directory
    std::fs::create_dir_all(&config.build.output_dir)?;

    let mut cmd = std::process::Command::new("wasm-bindgen");
    cmd.arg(&wasm_path)
        .arg("--out-dir")
        .arg(&config.build.output_dir)
        .arg("--target")
        .arg("web");

    if config.build.sourcemap {
        cmd.arg("--keep-debug");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(crate::TairitsuPackagerError::BuildError(
            "wasm-bindgen failed".to_string()
        ));
    }

    Ok(())
}

fn generate_html(config: &Config) -> crate::Result<()> {
    let pkg_name = &config.package.name;
    let js_file = format!("{}_bg.js", pkg_name.replace('-', "_"));

    let title = config.html.title.as_deref()
        .unwrap_or(&config.package.name);

    let html_content = format!(r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="{}">
    <meta name="viewport" content="{}">
    <title>{}</title>
    {}
</head>
<body class="{}">
    <div id="app">Loading...</div>
    <script type="module">
        import init from './{}';
        init();
    </script>
</body>
</html>"#,
        config.html.lang,
        config.html.charset,
        config.html.viewport,
        title,
        config.html.head,
        config.html.body_class,
        js_file,
    );

    let html_path = config.build.output_dir.join("index.html");
    std::fs::write(&html_path, html_content)?;

    Ok(())
}

pub async fn dev_server(config: &Config, port: u16, open: bool) -> crate::Result<()> {
    use axum::{
        routing::get,
        Router,
        response::Html,
    };
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Tairitsu Development Server");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    
    // Build WASM first
    println!("[1/3] Building WASM...");
    build(config, false)?;
    
    let dist_dir = config.build.output_dir.clone();
    
    println!("\n[2/3] Starting development server...");
    
    // Check if index.html exists
    let index_path = dist_dir.join("index.html");
    let index_content = if index_path.exists() {
        std::fs::read_to_string(&index_path)?
    } else {
        format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body><div id=\"app\">Loading...</div><script type=\"module\" src=\"./{}_bg.js\"></script></body></html>",
            config.package.name,
            config.package.name.replace('-', "_")
        )
    };
    
    // Setup static file server
    let index_html = index_content.clone();
    let app = Router::new()
        .route("/", get(move || async move { 
            Html(index_html.clone())
        }))
        .fallback_service(ServeDir::new(dist_dir));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("\n[3/3] Server ready!");
    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🌍 Local:   http://localhost:{}", port);
    println!("📁 Serving: {}", config.build.output_dir.display());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Press Ctrl+C to stop");
    
    // Open browser if requested
    if open {
        let url = format!("http://localhost:{}", port);
        match webbrowser::open(&url) {
            Ok(_) => println!("✓ Opening browser..."),
            Err(e) => eprintln!("⚠ Failed to open browser: {}", e),
        }
    }
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
