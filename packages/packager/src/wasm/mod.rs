use crate::config::Config;
use indicatif::{ProgressBar, ProgressStyle};

pub fn build(config: &Config, release: bool) -> crate::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );

    // Step 1: Check wasm32-unknown-unknown target
    pb.set_message("Checking WASM target...");
    check_wasm_target()?;

    // Step 2: Build WASM
    pb.set_message("Compiling WASM...");
    build_wasm(release)?;

    // Step 3: Run wasm-bindgen
    pb.set_message("Generating JS bindings...");
    run_wasm_bindgen(config, release)?;

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
    let current_manifest = std::env::current_dir()?.join("Cargo.toml");
    let content = std::fs::read_to_string(&current_manifest)?;
    let manifest: toml::Value = toml::from_str(&content)?;

    let pkg_name = manifest
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            crate::TairitsuPackagerError::BuildError(
                "Failed to read package name from Cargo.toml".to_string(),
            )
        })?;

    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        "wasm32-unknown-unknown",
        "--package",
        pkg_name,
    ]);

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(crate::TairitsuPackagerError::BuildError(
            "Cargo build failed".to_string(),
        ));
    }

    Ok(())
}

fn find_workspace_root() -> crate::Result<std::path::PathBuf> {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()?;

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let workspace_root = metadata
        .get("workspace_root")
        .and_then(|v| v.as_str())
        .map(std::path::PathBuf::from)
        .ok_or_else(|| {
            crate::TairitsuPackagerError::BuildError("Failed to find workspace root".to_string())
        })?;

    Ok(workspace_root)
}

fn run_wasm_bindgen(config: &Config, release: bool) -> crate::Result<()> {
    let workspace_root = find_workspace_root()?;
    let pkg_name = &config.package.name;
    let profile = if release { "release" } else { "debug" };
    let wasm_path = workspace_root
        .join("target")
        .join("wasm32-unknown-unknown")
        .join(profile)
        .join(format!("{}.wasm", pkg_name.replace('-', "_")));

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
            "wasm-bindgen failed".to_string(),
        ));
    }

    Ok(())
}

fn generate_html(config: &Config) -> crate::Result<()> {
    let pkg_name = &config.package.name;
    let js_file = format!("{}.js", pkg_name.replace('-', "_"));

    let title = config.html.title.as_deref().unwrap_or(&config.package.name);

    let html_content = format!(
        r#"<!DOCTYPE html>
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

/// 构建 wasm32-wasip2 格式的 WASM Component，输出格式与 wasm-bindgen 类似，
/// 但使用 WIT Component Model 及 tairitsu browser-glue 实现浏览器互操作。
///
/// 构建步骤：
/// 1. 检查 wasm32-wasip2 toolchain 已安装
/// 2. `cargo build --target wasm32-wasip2 --lib`
/// 3. 将 .wasm 组件拷贝到输出目录
/// 4. 将 browser-glue/dist/ 拷贝到输出目录
/// 5. 生成宿主 HTML（通过 browser-glue 加载组件）
pub fn build_component(config: &Config, release: bool) -> crate::Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Step 1: Check wasm32-wasip2 target
    pb.set_message("Checking wasm32-wasip2 target...");
    check_wasip2_target()?;

    // Step 2: Build WASM Component
    pb.set_message("Compiling WASM component (wasm32-wasip2)...");
    let wasm_path = build_wasm_component(config, release)?;

    // Step 3: Copy component to output dir
    pb.set_message("Copying WASM component...");
    std::fs::create_dir_all(&config.build.output_dir)?;
    let dest_wasm = config
        .build
        .output_dir
        .join(format!("{}.wasm", config.package.name.replace('-', "_")));
    std::fs::copy(&wasm_path, &dest_wasm)?;

    // Step 4: Bundle browser-glue
    pb.set_message("Bundling browser-glue...");
    copy_browser_glue(config)?;

    // Step 5: Copy static assets from ./public
    pb.set_message("Copying static assets...");
    copy_static_public_assets(config)?;

    // Step 6: Prepare wrapper fallback for browsers without native Component API
    pb.set_message("Preparing component wrapper fallback...");
    prepare_component_wrapper_fallback(config, &dest_wasm)?;

    // Step 7: Generate HTML
    pb.set_message("Generating HTML...");
    generate_component_html(config)?;

    pb.finish_with_message("Component build complete! ✅");
    println!("\nOutput: {}", config.build.output_dir.display());

    Ok(())
}

fn check_wasip2_target() -> crate::Result<()> {
    let output = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()?;

    let targets = String::from_utf8_lossy(&output.stdout);
    if !targets.contains("wasm32-wasip2") {
        return Err(crate::TairitsuPackagerError::BuildError(
            "wasm32-wasip2 target not installed. Run: rustup target add wasm32-wasip2".to_string(),
        ));
    }

    Ok(())
}

fn build_wasm_component(config: &Config, release: bool) -> crate::Result<std::path::PathBuf> {
    let pkg_name = &config.package.name;

    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        "wasm32-wasip2",
        "--lib",
        "--package",
        pkg_name,
    ]);
    if release {
        cmd.arg("--release");
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(crate::TairitsuPackagerError::BuildError(
            "cargo build --target wasm32-wasip2 failed".to_string(),
        ));
    }

    let workspace_root = find_workspace_root()?;
    let profile = if release { "release" } else { "debug" };
    let wasm_path = workspace_root
        .join("target")
        .join("wasm32-wasip2")
        .join(profile)
        .join(format!("{}.wasm", pkg_name.replace('-', "_")));

    if !wasm_path.exists() {
        return Err(crate::TairitsuPackagerError::BuildError(format!(
            "Expected component at {} but not found",
            wasm_path.display()
        )));
    }

    Ok(wasm_path)
}

fn copy_browser_glue(config: &Config) -> crate::Result<()> {
    let workspace_root = find_workspace_root()?;
    let glue_dist = workspace_root
        .join("packages")
        .join("browser-glue")
        .join("dist");

    if !glue_dist.exists() {
        eprintln!(
            "⚠  browser-glue/dist not found at {}.\n   \
             Run `npm run build` in packages/browser-glue/ first, \
             or the component will not have browser bindings.",
            glue_dist.display()
        );
        return Ok(());
    }

    let target_glue = config.build.output_dir.join("browser-glue");
    std::fs::create_dir_all(&target_glue)?;

    for entry in std::fs::read_dir(&glue_dist)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let dest = target_glue.join(path.file_name().unwrap());
            std::fs::copy(&path, &dest)?;
        }
    }

    Ok(())
}

fn copy_static_public_assets(config: &Config) -> crate::Result<()> {
    let project_root = std::env::current_dir()?;
    let public_dir = project_root.join("public");

    if !public_dir.exists() {
        return Ok(());
    }

    for entry in walkdir::WalkDir::new(&public_dir) {
        let entry = entry.map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!("Failed to walk public assets: {}", e))
        })?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let relative = path.strip_prefix(&public_dir).map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!(
                "Failed to compute public asset relative path: {}",
                e
            ))
        })?;
        let dest = config.build.output_dir.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(path, dest)?;
    }

    Ok(())
}

fn prepare_component_wrapper_fallback(
    config: &Config,
    component_wasm_path: &std::path::Path,
) -> crate::Result<()> {
    write_component_wrapper_loader(config, component_wasm_path)?;

    if !try_generate_component_wrapper(config, component_wasm_path)? {
        let wasm_hint_path = std::fs::canonicalize(component_wasm_path)
            .unwrap_or_else(|_| component_wasm_path.to_path_buf());
        let wasm_hint = wasm_hint_path
            .display()
            .to_string()
            .trim_start_matches(r"\\?\")
            .to_string();
        eprintln!(
            "⚠  Component wrapper was not generated automatically.\n   \
             Browsers without native WebAssembly Component API will still fail.\n   \
             See detailed diagnostics above for missing commands and command output.\n   \
             Quick manual command:\n   \
             npx --yes @bytecodealliance/jco transpile \"{}\" -o \"component-wrapper\"",
            wasm_hint
        );
    }

    // Always normalize wrapper imports if wrapper files already exist.
    rewrite_wrapper_imports_to_esm(config)?;

    Ok(())
}

fn rewrite_wrapper_imports_to_esm(config: &Config) -> crate::Result<()> {
    let wrapper_dir = config.build.output_dir.join("component-wrapper");
    if !wrapper_dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(&wrapper_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("js") {
            continue;
        }

        let content = std::fs::read_to_string(&path)?;
        let patched = content
            .replace(
                "'@bytecodealliance/preview2-shim/",
                "'https://esm.sh/@bytecodealliance/preview2-shim/",
            )
            .replace(
                "\"@bytecodealliance/preview2-shim/",
                "\"https://esm.sh/@bytecodealliance/preview2-shim/",
            );

        if patched != content {
            std::fs::write(&path, patched)?;
        }
    }

    Ok(())
}

fn write_component_wrapper_loader(
    config: &Config,
    component_wasm_path: &std::path::Path,
) -> crate::Result<()> {
    let wasm_stem = component_wasm_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            crate::TairitsuPackagerError::BuildError(
                "Failed to derive wasm file stem for wrapper loader".to_string(),
            )
        })?;

    let loader = format!(
        r#"export async function instantiateWithWrapper(imports = {{}}) {{
    const candidates = [
        './component-wrapper/{wasm_stem}.js',
        './component-wrapper/index.js',
    ];

    let lastError = null;

    for (const path of candidates) {{
        try {{
            const mod = await import(path);
            const instantiate = mod.instantiate || mod.default || mod.init;
            if (typeof instantiate !== 'function') {{
                // Some transpilers emit self-initializing modules (top-level await)
                // with no explicit instantiate export. Import success means ready.
                return mod;
            }}

            // Try common transpiler signatures in order.
            try {{
                return await instantiate(imports);
            }} catch (_e1) {{}}

            try {{
                return await instantiate(async (modulePath) => {{
                    const resolved = new URL(modulePath, import.meta.url);
                    const response = await fetch(resolved);
                    if (!response.ok) {{
                        throw new Error(`Failed to fetch core module: ${{modulePath}}`);
                    }}
                    return WebAssembly.compileStreaming(response);
                }}, imports);
            }} catch (_e2) {{}}
        }} catch (e) {{
            lastError = e;
        }}
    }}

    throw new Error(
        'Component wrapper not found or could not be initialized. '
        + 'Expected a transpiled wrapper under ./component-wrapper/. '
        + (lastError ? `Last error: ${{lastError}}` : '')
    );
}}
"#,
        wasm_stem = wasm_stem
    );

    let loader_path = config.build.output_dir.join("component-wrapper-loader.js");
    std::fs::write(loader_path, loader)?;
    Ok(())
}

fn try_generate_component_wrapper(
    config: &Config,
    component_wasm_path: &std::path::Path,
) -> crate::Result<bool> {
    let wrapper_dir = config.build.output_dir.join("component-wrapper");
    std::fs::create_dir_all(&wrapper_dir)?;

    let wasm_path_for_cmd = std::fs::canonicalize(component_wasm_path)
        .unwrap_or_else(|_| component_wasm_path.to_path_buf());
    let wrapper_dir_for_cmd = std::fs::canonicalize(&wrapper_dir).unwrap_or(wrapper_dir.clone());
    let wasm_path_hint = wasm_path_for_cmd
        .display()
        .to_string()
        .trim_start_matches(r"\\?\")
        .to_string();
    let wrapper_dir_hint = wrapper_dir_for_cmd
        .display()
        .to_string()
        .trim_start_matches(r"\\?\")
        .to_string();

    let wasm_stem = component_wasm_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| {
            crate::TairitsuPackagerError::BuildError(
                "Failed to derive wasm file stem for wrapper detection".to_string(),
            )
        })?;

    let wrapper_main = if wrapper_dir.join("index.js").exists() {
        Some(wrapper_dir.join("index.js"))
    } else {
        let named = wrapper_dir.join(format!("{}.js", wasm_stem));
        if named.exists() {
            Some(named)
        } else {
            None
        }
    };

    if let Some(main) = wrapper_main {
        let wrapper_mtime = std::fs::metadata(&main)?.modified().ok();
        let wasm_mtime = std::fs::metadata(component_wasm_path)?.modified().ok();
        if let (Some(w), Some(c)) = (wrapper_mtime, wasm_mtime) {
            if w >= c {
                return Ok(true);
            }
        }
    }

    let attempts: [(&str, Vec<String>); 2] = [
        (
            "jco",
            vec![
                "transpile".to_string(),
                wasm_path_for_cmd.display().to_string(),
                "-o".to_string(),
                wrapper_dir_for_cmd.display().to_string(),
            ],
        ),
        (
            "npx",
            vec![
                "--yes".to_string(),
                "@bytecodealliance/jco".to_string(),
                "transpile".to_string(),
                wasm_path_for_cmd.display().to_string(),
                "-o".to_string(),
                wrapper_dir_for_cmd.display().to_string(),
            ],
        ),
    ];

    let mut missing_commands = Vec::new();

    for (bin, args) in attempts {
        let command_preview = format!("{} {}", bin, args.join(" "));
        match std::process::Command::new(bin).args(&args).output() {
            Ok(output) if output.status.success() => {
                let has_index = wrapper_dir.join("index.js").exists();
                let has_named = wrapper_dir.join(format!("{}.js", wasm_stem)).exists();
                if has_index || has_named {
                    return Ok(true);
                }
                eprintln!(
                    "⚠  Wrapper transpile command succeeded but no JS wrapper entry was found.\n   Command: {}",
                    command_preview
                );
                continue;
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let detail = if !stderr.is_empty() {
                    stderr
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    "(no output)".to_string()
                };
                let detail_preview: String = detail.chars().take(400).collect();

                eprintln!(
                    "⚠  Wrapper transpile command failed.\n   Command: {}\n   Exit: {}\n   Detail: {}",
                    command_preview,
                    output.status,
                    detail_preview
                );
                continue;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                missing_commands.push(bin.to_string());
                eprintln!(
                    "⚠  Wrapper transpile command not found: '{}'.\n   Attempted: {}",
                    bin, command_preview
                );
                continue;
            }
            Err(err) => {
                eprintln!(
                    "⚠  Failed to execute wrapper transpile command.\n   Command: {}\n   Error: {}",
                    command_preview, err
                );
                continue;
            }
        }
    }

    if !missing_commands.is_empty() {
        missing_commands.sort();
        missing_commands.dedup();
        eprintln!(
            "⚠  Missing wrapper tooling in PATH: {}\n   Install Node.js + JCO or ensure these commands are available.",
            missing_commands.join(", ")
        );
    }

    eprintln!(
        "ℹ  Manual fallback command (absolute paths):\n   npx --yes @bytecodealliance/jco transpile \"{}\" -o \"{}\"",
        wasm_path_hint,
        wrapper_dir_hint
    );

    Ok(false)
}

fn generate_component_html(config: &Config) -> crate::Result<()> {
    let pkg_name = &config.package.name;
    let wasm_file = format!("{}.wasm", pkg_name.replace('-', "_"));

    let title = config.html.title.as_deref().unwrap_or(pkg_name.as_str());
    let favicon = config
        .html
        .favicon
        .as_deref()
        .map(|f| f.strip_prefix("public/").unwrap_or(f).replace('\\', "/"));
    let favicon_link = favicon
        .as_deref()
        .map(|href| format!("<link rel=\"icon\" href=\"./{}\">", href))
        .unwrap_or_default();

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="{charset}">
    <meta name="viewport" content="{viewport}">
    <title>{title}</title>
    {favicon_link}
    {head}
</head>
<body class="{body_class}">
    <div id="app">Loading...</div>
        <script type="importmap">
        {{
            "imports": {{
                "@bytecodealliance/preview2-shim/": "https://esm.sh/@bytecodealliance/preview2-shim/",
                "tairitsu-browser:full/node": "./browser-glue/dom-glue.js",
                "tairitsu-browser:full/document": "./browser-glue/dom-glue.js",
                "tairitsu-browser:full/window": "./browser-glue/dom-glue.js",
                "tairitsu-browser:full/style": "./browser-glue/dom-glue.js",
                "tairitsu-browser:full/event-target": "./browser-glue/events-glue.js",
                "tairitsu-browser:full/fetch-api": "./browser-glue/fetch-glue.js",
                "tairitsu-browser:full/canvas2d": "./browser-glue/canvas-glue.js"
            }}
        }}
        </script>
    <script type="module">
        // Load browser API glue (WIT interface implementations)
        import './browser-glue/index.js';
        import {{ instantiateWithWrapper }} from './component-wrapper-loader.js';

        const appRoot = document.getElementById('app');
        const setAppStatus = (text) => {{
            if (!appRoot) return;
            const current = (appRoot.textContent || '').trim();
            if (current === 'Loading...') {{
                appRoot.textContent = text;
            }}
        }};

        const clearLoadingIfUnchanged = () => {{
            if (!appRoot) return;
            const current = (appRoot.textContent || '').trim();
            if (current === 'Loading...') {{
                appRoot.textContent = '';
            }}
        }};

        const tryInvokeBootExports = async (result) => {{
            const isBootName = (name) => /^(run|start|init|main)$/i.test(name);
            const seen = new Set();

            const walk = async (obj, depth = 0) => {{
                if (!obj || typeof obj !== 'object' || depth > 3) return false;
                if (seen.has(obj)) return false;
                seen.add(obj);

                for (const [name, value] of Object.entries(obj)) {{
                    if (typeof value === 'function' && isBootName(name)) {{
                        await value();
                        return true;
                    }}
                }}

                for (const [, value] of Object.entries(obj)) {{
                    if (value && typeof value === 'object') {{
                        const ok = await walk(value, depth + 1);
                        if (ok) return true;
                    }}
                }}

                return false;
            }};

            const targets = [
                result,
                result && result.instance,
                result && result.exports,
                result && result.instance && result.instance.exports,
            ];

            for (const target of targets) {{
                if (await walk(target)) return true;
                if (target && target.exports && await walk(target.exports)) return true;
            }}

            return false;
        }};

        // Load wasm bytes and detect whether this is a Component or core module
        const response = await fetch('./{wasm_file}');
        const bytes = await response.arrayBuffer();
        const magic = new Uint8Array(bytes, 0, 8);
        const isWasm =
            magic[0] === 0x00 && magic[1] === 0x61 && magic[2] === 0x73 && magic[3] === 0x6d;
        const isComponent = isWasm &&
            magic[4] === 0x0d && magic[5] === 0x00 && magic[6] === 0x01 && magic[7] === 0x00;

        let bootInvoked = false;

        if (isComponent) {{
            if (typeof WebAssembly.Component !== 'function') {{
                const wrapperResult = await instantiateWithWrapper({{}});
                bootInvoked = await tryInvokeBootExports(wrapperResult);
            }} else {{
                const component = new WebAssembly.Component(bytes);
                const componentResult = await WebAssembly.instantiate(component, {{}});
                bootInvoked = await tryInvokeBootExports(componentResult);
            }}
        }} else {{
            const module = await WebAssembly.compile(bytes);
            const moduleResult = await WebAssembly.instantiate(module, {{}});
            bootInvoked = await tryInvokeBootExports(moduleResult);
        }}

        if (!bootInvoked) {{
            setAppStatus('Component initialized (no exported run/start entry found).');
        }} else {{
            clearLoadingIfUnchanged();
        }}
    </script>
</body>
</html>"#,
        lang = config.html.lang,
        charset = config.html.charset,
        viewport = config.html.viewport,
        title = title,
        favicon_link = favicon_link,
        head = config.html.head,
        body_class = config.html.body_class,
        wasm_file = wasm_file,
    );

    let html_path = config.build.output_dir.join("index.html");
    std::fs::write(&html_path, html)?;

    Ok(())
}

pub async fn dev_server(config: &Config, port: u16, open: bool) -> crate::Result<()> {
    use axum::{response::Html, routing::get, Router};
    use std::net::SocketAddr;
    use tower_http::services::ServeDir;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Tairitsu Development Server");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Build first, following project target preference.
    println!("[1/3] Building project artifacts...");
    match config.build.target.as_str() {
        "component" => build_component(config, false)?,
        "wasm" => build(config, false)?,
        other => {
            return Err(crate::TairitsuPackagerError::BuildError(format!(
                "Unknown build target '{}'. Use 'wasm' or 'component'.",
                other
            )));
        }
    }

    let dist_dir = config.build.output_dir.clone();

    println!("\n[2/3] Starting development server...");

    // Check if index.html exists
    let index_path = dist_dir.join("index.html");
    let index_content = if index_path.exists() {
        std::fs::read_to_string(&index_path)?
    } else {
        format!(
            "<!DOCTYPE html><html><head><title>{}</title></head><body><div id=\"app\">Loading...</div><script type=\"module\" src=\"./{}.js\"></script></body></html>",
            config.package.name,
            config.package.name.replace('-', "_")
        )
    };

    // Setup static file server
    let index_html = index_content.clone();
    let app = Router::new()
        .route("/", get(move || async move { Html(index_html.clone()) }))
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
    if open || config.dev.open_browser {
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
