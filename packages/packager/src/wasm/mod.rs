use crate::config::Config;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::time::{Duration, Instant};

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
    let build_start = Instant::now();
    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::with_template("  {spinner:.bold.cyan}  {prefix:.bold.dim}  {wide_msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.enable_steady_tick(Duration::from_millis(80));

    // ── 1/5  check target ─────────────────────────────────────────────────────
    pb.set_prefix("[1/5]");
    pb.set_message("check wasm32-wasip2");
    let t = Instant::now();
    check_wasip2_target()?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "check wasm32-wasip2",
        t.elapsed()
    ));
    pb.inc(1);

    // ── 2/5  compile ──────────────────────────────────────────────────────────
    pb.set_prefix("[2/5]");
    pb.set_message("compile WASM component");
    let t = Instant::now();
    let wasm_path = build_wasm_component(config, release)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "compile WASM component",
        t.elapsed()
    ));
    pb.inc(1);

    // ── 3/5  bundle assets ────────────────────────────────────────────────────
    pb.set_prefix("[3/5]");
    pb.set_message("bundle assets");
    let t = Instant::now();
    let dest_wasm = config
        .build
        .output_dir
        .join(format!("{}.wasm", config.package.name.replace('-', "_")));
    std::fs::create_dir_all(&config.build.output_dir)?;
    std::fs::copy(&wasm_path, &dest_wasm)?;
    copy_browser_glue(config)?;
    copy_static_public_assets(config)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "bundle assets",
        t.elapsed()
    ));
    pb.inc(1);

    // ── 4/5  component wrapper ────────────────────────────────────────────────
    pb.set_prefix("[4/5]");
    pb.set_message("component wrapper");
    let t = Instant::now();
    prepare_component_wrapper_fallback(config, &dest_wasm)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "component wrapper",
        t.elapsed()
    ));
    pb.inc(1);

    // ── 5/5  HTML ─────────────────────────────────────────────────────────────
    pb.set_prefix("[5/5]");
    pb.set_message("generate HTML");
    let t = Instant::now();
    generate_component_html(config)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "generate HTML",
        t.elapsed()
    ));
    pb.inc(1);

    pb.finish_and_clear();
    println!();
    println!(
        "  ✓  build complete  {:.1?}    📦  {}",
        build_start.elapsed(),
        config.build.output_dir.display()
    );
    println!();

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

    let loader = include_str!(concat!(
        env!("OUT_DIR"),
        "/component-wrapper-loader.template.js"
    ))
    .replace("__WASM_STEM__", wasm_stem);

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

    let mut attempts: Vec<(&str, Vec<String>)> = vec![
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

    #[cfg(windows)]
    {
        attempts.push((
            "npx.cmd",
            vec![
                "--yes".to_string(),
                "@bytecodealliance/jco".to_string(),
                "transpile".to_string(),
                wasm_path_for_cmd.display().to_string(),
                "-o".to_string(),
                wrapper_dir_for_cmd.display().to_string(),
            ],
        ));
    }

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
    // Build-time version stamp so browsers never serve stale cached JS/WASM
    // modules even if `Cache-Control: no-store` is bypassed by a caching proxy.
    let v = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

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
                "tairitsu-browser:full/node": "./browser-glue/dom-glue.js?v={v}",
                "tairitsu-browser:full/document": "./browser-glue/dom-glue.js?v={v}",
                "tairitsu-browser:full/window": "./browser-glue/dom-glue.js?v={v}",
                "tairitsu-browser:full/style": "./browser-glue/dom-glue.js?v={v}",
                "tairitsu-browser:full/event-target": "./browser-glue/events-glue.js?v={v}",
                "tairitsu-browser:full/fetch-api": "./browser-glue/fetch-glue.js?v={v}",
                "tairitsu-browser:full/canvas2d": "./browser-glue/canvas-glue.js?v={v}"
            }}
        }}
        </script>
    <script type="module">
        // Load browser API glue (WIT interface implementations)
        import './browser-glue/index.js?v={v}';
        import {{ instantiateWithWrapper }} from './component-wrapper-loader.js?v={v}';

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
        v = v,
    );

    let html_path = config.build.output_dir.join("index.html");
    std::fs::write(&html_path, html)?;

    Ok(())
}

/// Middleware that prevents browsers from caching JS/WASM/HTML assets in dev mode.
/// Without this, browsers serve stale cached modules even after a file changes.
async fn no_cache_headers(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;

    let should_bust = path == "/"
        || path.ends_with(".html")
        || path.ends_with(".js")
        || path.ends_with(".mjs")
        || path.ends_with(".wasm")
        || path.ends_with(".map");

    if should_bust {
        response.headers_mut().insert(
            axum::http::header::CACHE_CONTROL,
            axum::http::HeaderValue::from_static("no-store"),
        );
    }
    response
}

pub async fn dev_server(config: &Config, port: u16, open: bool, watch: bool) -> crate::Result<()> {
    use axum::{middleware, response::Html, routing::get, Router};
    use tower_http::services::ServeDir;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if watch {
        println!("  Tairitsu  ↻  Development  (watch mode)");
    } else {
        println!("  Tairitsu  Development Server");
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Initial build.
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

    // Always read index.html from disk on each request so watch-mode rebuilds
    // are served immediately without needing a server restart.
    let dist_for_index = dist_dir.clone();
    let pkg_name = config.package.name.clone();
    let app = Router::new()
        .route(
            "/",
            get(move || {
                let dist = dist_for_index.clone();
                let pkg = pkg_name.clone();
                async move {
                    let content =
                        std::fs::read_to_string(dist.join("index.html")).unwrap_or_else(|_| {
                            format!(
                                "<!DOCTYPE html><html><head><title>{}</title></head>\
                                 <body><div id=\"app\">Loading…</div></body></html>",
                                pkg
                            )
                        });
                    Html(content)
                }
            }),
        )
        .fallback_service(ServeDir::new(dist_dir))
        .layer(middleware::from_fn(no_cache_headers));

    let (listener, actual_port) = bind_listener_with_fallback(port).await?;
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  🌍  Local:   http://localhost:{}", actual_port);
    println!("  📁  Serving: {}", config.build.output_dir.display());
    if actual_port != port {
        println!(
            "  ⚠   Port {} is busy, switched to {}",
            port, actual_port
        );
    }
    if watch {
        println!("  👁   Watch:  src/  Cargo.toml  public/");
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    if open || config.dev.open_browser {
        let url = format!("http://localhost:{}", actual_port);
        match webbrowser::open(&url) {
            Ok(_) => println!("  ✓  Opening browser…"),
            Err(e) => eprintln!("  ⚠  Failed to open browser: {}", e),
        }
    }

    if watch {
        let ui = DevWatchUi::new(actual_port, &config.build.output_dir, &config.build.target);
        // Run the HTTP server as a background task; this task runs the watch loop.
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        run_watch_loop(config, actual_port, Some(ui)).await?;
    } else {
        println!("  Press Ctrl+C to stop");
        axum::serve(listener, app).await?;
    }

    Ok(())
}

/// Try binding localhost starting from `preferred_port` and automatically
/// fallback to higher ports when the preferred one is already occupied.
async fn bind_listener_with_fallback(
    preferred_port: u16,
) -> crate::Result<(tokio::net::TcpListener, u16)> {
    const MAX_CANDIDATES: u16 = 20;

    for offset in 0..MAX_CANDIDATES {
        let candidate = preferred_port.saturating_add(offset);
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], candidate));

        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok((listener, candidate)),
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(err) => {
                return Err(crate::TairitsuPackagerError::BuildError(format!(
                    "failed to bind dev server at {}: {}",
                    addr, err
                )));
            }
        }
    }

    let last = preferred_port.saturating_add(MAX_CANDIDATES - 1);
    Err(crate::TairitsuPackagerError::BuildError(format!(
        "failed to bind dev server: no free port in range {}..={}",
        preferred_port, last
    )))
}

#[derive(Clone)]
struct DevWatchUi {
    _multi: std::sync::Arc<MultiProgress>,
    server: ProgressBar,
    watcher: ProgressBar,
    build: ProgressBar,
    started: Instant,
    port: u16,
}

impl DevWatchUi {
    fn new(port: u16, output_dir: &std::path::Path, target: &str) -> Self {
        let multi = std::sync::Arc::new(MultiProgress::new());

        let style = ProgressStyle::with_template("  {spinner:.green} {msg}").unwrap();
        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        let server = multi.add(ProgressBar::new_spinner());
        server.set_style(style.clone().tick_strings(&spinner_frames));
        server.enable_steady_tick(Duration::from_millis(90));
        server.set_message(format!(
            "serve  http://localhost:{}   ({})",
            port,
            output_dir.display()
        ));

        let watcher = multi.add(ProgressBar::new_spinner());
        watcher.set_style(style.clone().tick_strings(&spinner_frames));
        watcher.enable_steady_tick(Duration::from_millis(110));
        watcher.set_message("watch  src/, Cargo.toml, Tairitsu.toml, public/");

        let build = multi.add(ProgressBar::new_spinner());
        build.set_style(style.tick_strings(&spinner_frames));
        build.enable_steady_tick(Duration::from_millis(130));
        build.set_message(format!("build  idle  (target: {})", target));

        Self {
            _multi: multi,
            server,
            watcher,
            build,
            started: Instant::now(),
            port,
        }
    }

    fn on_change(&self, changed: &[std::path::PathBuf]) {
        let msg = match changed.len() {
            0 => "watch  source changed".to_string(),
            1 => format!("watch  changed: {}", changed[0].display()),
            n => format!("watch  changed: {} files", n),
        };
        self.watcher.set_message(msg);
    }

    fn on_build_start(&self) {
        self.build.enable_steady_tick(Duration::from_millis(100));
        self.build.set_message("build  rebuilding...");
    }

    fn on_build_finish(&self, ok: bool, elapsed: Duration) {
        self.build.disable_steady_tick();
        let status = if ok { "ok" } else { "failed" };
        self.build.set_message(format!(
            "build  {}  in {:.1?}   |   uptime {:.1?}",
            status,
            elapsed,
            self.started.elapsed()
        ));
        if ok {
            self.server
                .set_message(format!("serve  http://localhost:{}   (hot)", self.port));
        }
    }
}

/// Watches `src/`, `Cargo.toml`, `Tairitsu.toml`, and `public/` for changes and
/// triggers incremental rebuilds using the same pipeline as the initial build.
///
/// Debounces rapid saves (200 ms window) so a single `cargo save` operation does
/// not trigger multiple concurrent builds.
async fn run_watch_loop(config: &Config, port: u16, ui: Option<DevWatchUi>) -> crate::Result<()> {
    use notify::{EventKind, RecursiveMode, Watcher};

    let project_root = std::env::current_dir()?;

    // Collect existing watch paths before moving them into the OS thread.
    let watch_paths: Vec<std::path::PathBuf> = [
        project_root.join("src"),
        project_root.join("Cargo.toml"),
        project_root.join("Tairitsu.toml"),
        project_root.join("public"),
    ]
    .into_iter()
    .filter(|p| p.exists())
    .collect();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<notify::Result<notify::Event>>(64);

    // Spawn the watcher on a dedicated OS thread.  Some platform backends
    // (e.g. FSEvents on macOS) are not Send; isolating them here avoids that.
    std::thread::spawn(move || {
        let Ok(mut watcher) = notify::recommended_watcher(move |ev| {
            tx.blocking_send(ev).ok();
        }) else {
            eprintln!("  ⚠  Failed to create file watcher");
            return;
        };

        for path in &watch_paths {
            let mode = if path.is_dir() {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            if let Err(e) = watcher.watch(path, mode) {
                eprintln!("  ⚠  Cannot watch {}: {}", path.display(), e);
            }
        }

        println!("  ↻  Watching for changes…  (Ctrl+C to stop)");

        // Keep the watcher alive until the process exits.
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });

    let debounce = tokio::time::Duration::from_millis(200);

    loop {
        // Wait for the first filesystem event.
        let event = match rx.recv().await {
            None => break, // channel closed (watcher thread exited)
            Some(Err(e)) => {
                eprintln!("  ⚠  Watch error: {}", e);
                continue;
            }
            Some(Ok(ev)) => ev,
        };

        // Only react to create / modify / remove events.
        if !matches!(
            event.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            continue;
        }

        // Collect the triggering paths, then drain further events within the
        // debounce window so rapid multi-file saves cause only one rebuild.
        let mut changed: Vec<std::path::PathBuf> = event
            .paths
            .iter()
            .filter_map(|p| p.strip_prefix(&project_root).ok().map(|r| r.to_path_buf()))
            .collect();

        let deadline = tokio::time::Instant::now() + debounce;
        loop {
            match tokio::time::timeout_at(deadline, rx.recv()).await {
                Ok(Some(Ok(ev))) => {
                    changed.extend(ev.paths.iter().filter_map(|p| {
                        p.strip_prefix(&project_root).ok().map(|r| r.to_path_buf())
                    }))
                }
                _ => break,
            }
        }
        changed.sort();
        changed.dedup();

        if let Some(ui) = &ui {
            ui.on_change(&changed);
            ui.on_build_start();
        } else {
            println!();
            match changed.len() {
                0 => println!("  ↻  source changed"),
                1 => println!("  ↻  {}", changed[0].display()),
                n => println!("  ↻  {} files changed", n),
            }
            println!();
        }

        // Run the rebuild on a blocking thread so the tokio runtime stays alive.
        let rebuild_started = Instant::now();
        let config_clone = config.clone();
        let target = config.build.target.clone();
        let result = tokio::task::spawn_blocking(move || match target.as_str() {
            "component" => build_component(&config_clone, false),
            _ => build(&config_clone, false),
        })
        .await
        .map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!("rebuild task panicked: {}", e))
        })?;

        let elapsed = rebuild_started.elapsed();
        match result {
            Ok(()) => {
                if let Some(ui) = &ui {
                    ui.on_build_finish(true, elapsed);
                } else {
                    println!("  ✓  rebuilt  →  http://localhost:{}", port);
                }
            }
            Err(e) => {
                if let Some(ui) = &ui {
                    ui.on_build_finish(false, elapsed);
                }
                eprintln!("  ✗  {}", e);
            }
        }
    }

    Ok(())
}
