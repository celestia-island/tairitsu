use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::config::Config;

fn locale() -> &'static crate::i18n::Translations {
    crate::i18n::translations()
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

/// 构建 wasm32-wasip2 格式的 WASM Component，
/// 但使用 WIT Component Model 及 tairitsu browser-glue 实现浏览器互操作。
///
/// 构建步骤：
/// 1. 检查 wasm32-wasip2 toolchain 已安装
/// 2. `cargo build --target wasm32-wasip2 --lib`
/// 3. 将 .wasm 组件拷贝到输出目录
/// 4. 将 browser-glue/dist/ 拷贝到输出目录
/// 5. 生成宿主 HTML（通过 browser-glue 加载组件）
pub fn build_component(
    config: &Config,
    release: bool,
    multi: Option<std::sync::Arc<MultiProgress>>,
) -> crate::Result<()> {
    let build_start = Instant::now();
    let pb_raw = ProgressBar::new(5);
    pb_raw.set_style(
        ProgressStyle::with_template("  {spinner:.bold.cyan}  {prefix:.bold.dim}  {wide_msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb_raw.enable_steady_tick(Duration::from_millis(80));
    // Attach to MultiProgress when in watch mode so pb.println() lines
    // scroll above the persistent status bars instead of below them.
    let pb = match &multi {
        Some(m) => m.add(pb_raw),
        None => pb_raw,
    };

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
    // Pass the progress bar so cargo compilation can update it directly
    let wasm_path = build_wasm_component(config, release, pb.clone())?;
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
    copy_browser_glue(config, &pb)?;
    copy_static_public_assets(config)?;
    compile_project_scss(config)?;
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
    prepare_component_wrapper_fallback(config, &dest_wasm, &pb)?;
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

fn build_wasm_component(
    config: &Config,
    release: bool,
    pb: ProgressBar,
) -> crate::Result<std::path::PathBuf> {
    use std::io::BufRead;
    use std::process::Stdio;

    let pkg_name = &config.package.name;

    // Use JSON message format to get structured compiler output.
    // This allows us to intercept cargo's progress and render our own.
    let mut cmd = std::process::Command::new("cargo");
    cmd.args([
        "build",
        "--target",
        "wasm32-wasip2",
        "--lib",
        "--package",
        pkg_name,
        "--message-format=json-diagnostic-rendered-ansi",
    ]);
    if release {
        cmd.arg("--release");
    }

    // Capture stdout (JSON messages) and suppress stderr (cargo's native progress bars)
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null()); // Suppress cargo's native progress bars entirely

    let mut child = cmd.spawn()?;

    let stdout = child.stdout.take().expect("stdout should be piped");

    // Clone the progress bar for the thread
    let pb_clone = pb.clone();

    // Helper: extract crate name from package_id
    // Format: "registry+https://github.com/rust-lang/crates.io-index#name@version"
    // or: "path+file:///path#name@version"
    fn extract_crate_name(package_id: &str) -> &str {
        // Try to find the part after '#' and before '@'
        if let Some(after_hash) = package_id.split('#').nth(1) {
            if let Some(name) = after_hash.split('@').next() {
                return name;
            }
        }
        // Fallback: return the whole thing
        package_id
    }

    // Thread: parse JSON messages from stdout and update the progress bar
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let Ok(line) = line else { continue };
            // Try to parse as cargo JSON message
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
                if let Some(reason) = msg.get("reason").and_then(|r| r.as_str()) {
                    match reason {
                        "compiler-artifact" => {
                            // A crate is being compiled
                            if let Some(package_id) = msg.get("package_id").and_then(|p| p.as_str())
                            {
                                let crate_name = extract_crate_name(package_id);
                                // Only update for lib crates (not build scripts)
                                if let Some(target) = msg.get("target") {
                                    if target
                                        .get("kind")
                                        .and_then(|k| k.as_array())
                                        .is_some_and(|k| {
                                            k.iter().any(|kind| kind.as_str() == Some("lib"))
                                        })
                                    {
                                        pb_clone.set_message(format!("compile {}", crate_name));
                                    }
                                }
                            }
                        }
                        "compiler-message" => {
                            // This contains actual compiler output (errors, warnings)
                            if let Some(rendered) = msg
                                .get("message")
                                .and_then(|m| m.get("rendered"))
                                .and_then(|r| r.as_str())
                            {
                                // Print the rendered diagnostic above the progress bar
                                pb_clone.println(rendered);
                            }
                        }
                        "build-script-executed" => {
                            if let Some(package_id) = msg.get("package_id").and_then(|p| p.as_str())
                            {
                                let crate_name = extract_crate_name(package_id);
                                pb_clone.set_message(format!("build script {}", crate_name));
                            }
                        }
                        "build-finished" => {
                            // Keep the original message
                        }
                        _ => {}
                    }
                }
            }
        }
    });

    let status = child.wait()?;
    // Don't finish/clear here - the caller manages the progress bar lifecycle

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

/// Embedded browser-glue files compiled into the packager binary.
/// This ensures the packager works standalone without requiring external browser-glue.
static BROWSER_GLUE_DIST: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../browser-glue/dist");

fn copy_browser_glue(config: &Config, pb: &ProgressBar) -> crate::Result<()> {
    let target_glue = config.build.output_dir.join("browser-glue");
    std::fs::create_dir_all(&target_glue)?;

    // Strategy 1: Try workspace-relative path (for monorepo development)
    if let Ok(workspace_root) = find_workspace_root() {
        let glue_dist = workspace_root
            .join("packages")
            .join("browser-glue")
            .join("dist");

        if glue_dist.exists() {
            for entry in std::fs::read_dir(&glue_dist)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let dest = target_glue.join(path.file_name().unwrap());
                    std::fs::copy(&path, &dest)?;
                }
            }
            return Ok(());
        }
    }

    // Strategy 2: Use embedded browser-glue from compile time
    let mut file_count = 0;
    for file in BROWSER_GLUE_DIST.files() {
        let dest = target_glue.join(file.path().file_name().unwrap());
        std::fs::write(&dest, file.contents())?;
        file_count += 1;
    }

    if file_count == 0 {
        pb.println(
            "⚠  No browser-glue files found (neither on filesystem nor embedded).",
        );
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

fn compile_project_scss(config: &Config) -> crate::Result<()> {
    let project_root = std::env::current_dir()?;

    // Use new SCSS configuration system
    let results = crate::styles::compile_scss_with_config(
        &config.scss,
        &project_root,
        &config.build.output_dir,
    )
    .map_err(|e| {
        crate::TairitsuPackagerError::BuildError(format!("Failed to compile SCSS: {}", e))
    })?;

    // Log compiled files
    for result in results {
        tracing::debug!("Compiled SCSS: {}", result.output_path.display());
    }

    Ok(())
}

fn prepare_component_wrapper_fallback(
    config: &Config,
    component_wasm_path: &std::path::Path,
    pb: &ProgressBar,
) -> crate::Result<()> {
    write_component_wrapper_loader(config, component_wasm_path)?;

    if !try_generate_component_wrapper(config, component_wasm_path, pb)? {
        let wasm_hint_path = std::fs::canonicalize(component_wasm_path)
            .unwrap_or_else(|_| component_wasm_path.to_path_buf());
        let wasm_hint = wasm_hint_path
            .display()
            .to_string()
            .trim_start_matches(r"\\?\")
            .to_string();
        pb.println(format!(
            "⚠  Wrapper transpile command not found: 'jco'.\n   \
             Attempted: jco transpile {} -o {}",
            wasm_hint,
            config.build.output_dir.join("component-wrapper").display()
        ));
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
        env!("CARGO_MANIFEST_DIR"),
        "/src/wasm/component-wrapper-loader.template.ts"
    ))
    .replace("__WASM_STEM__", wasm_stem);

    let loader_path = config.build.output_dir.join("component-wrapper-loader.js");
    std::fs::write(loader_path, loader)?;
    Ok(())
}

fn try_generate_component_wrapper(
    config: &Config,
    component_wasm_path: &std::path::Path,
    pb: &ProgressBar,
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

    #[allow(unused_mut)]
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
                pb.println(format!(
                    "⚠  Wrapper transpile command succeeded but no JS wrapper entry was found.\n   Command: {}",
                    command_preview
                ));
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

                pb.println(format!(
                    "⚠  Wrapper transpile command failed.\n   Command: {}\n   Exit: {}\n   Detail: {}",
                    command_preview,
                    output.status,
                    detail_preview
                ));
                continue;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                missing_commands.push(bin.to_string());
                // Don't print per-command not found - we'll summarize at the end
                continue;
            }
            Err(err) => {
                pb.println(format!(
                    "⚠  Failed to execute wrapper transpile command.\n   Command: {}\n   Error: {}",
                    command_preview, err
                ));
                continue;
            }
        }
    }

    // Only print one summary message for missing commands
    if !missing_commands.is_empty() {
        missing_commands.sort();
        missing_commands.dedup();
        pb.println(format!(
            "⚠  Wrapper transpile command not found: '{}'.\n   \
             Attempted: jco transpile {} -o {}",
            missing_commands.join(", "),
            wasm_path_hint,
            wrapper_dir_hint
        ));
    }

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
                "tairitsu-browser:full/": "./browser-glue/"
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
            const normalizeBootName = (name) => {{
                const lowered = String(name || '').toLowerCase();
                if (lowered === 'run') return 'run';
                if (lowered === 'main') return 'main';
                if (lowered === 'init') return 'init';
                if (lowered === 'start') return 'start';
                return null;
            }};

            const seenObjects = new Set();
            const seenFunctions = new Set();
            const discovered = [];

            const collect = (obj, depth = 0) => {{
                if (!obj || typeof obj !== 'object' || depth > 3) return;
                if (seenObjects.has(obj)) return;
                seenObjects.add(obj);

                for (const [name, value] of Object.entries(obj)) {{
                    if (typeof value !== 'function') continue;
                    const kind = normalizeBootName(name);
                    if (!kind) continue;
                    if (seenFunctions.has(value)) continue;
                    seenFunctions.add(value);
                    discovered.push({{ kind, fn: value }});
                }}

                for (const [, value] of Object.entries(obj)) {{
                    if (value && typeof value === 'object') {{
                        collect(value, depth + 1);
                    }}
                }}
            }};

            const targets = [
                result,
                result && result.instance,
                result && result.exports,
                result && result.instance && result.instance.exports,
            ];

            for (const target of targets) {{
                collect(target);
                if (target && target.exports) collect(target.exports);
            }}

            let invoked = false;

            for (const preferred of ['run', 'main', 'init']) {{
                for (const entry of discovered) {{
                    if (entry.kind !== preferred) continue;
                    await entry.fn();
                    invoked = true;
                }}
            }}

            if (!invoked) {{
                const fallbackStart = discovered.find((entry) => entry.kind === 'start');
                if (fallbackStart) {{
                    await fallbackStart.fn();
                    invoked = true;
                }}
            }}

            return invoked;
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

    let divider = panel_divider();
    println!("{}", divider);
    if watch {
        println!("  Tairitsu  ↻  Development  (watch mode)");
    } else {
        println!("  Tairitsu  Development Server");
    }
    println!("{}", divider);
    println!();

    // Initial build (no MultiProgress — let cargo write directly to terminal).
    let initial_started = Instant::now();
    match config.build.target.as_str() {
        "component" => build_component(config, false, None)?,
        other => {
            return Err(crate::TairitsuPackagerError::BuildError(format!(
                "Unknown build target '{}'. Only 'component' is supported.",
                other
            )));
        }
    }
    let initial_elapsed = initial_started.elapsed();

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
    let mut last_build_line = format_last_build_line(true, initial_elapsed, None);

    let port_switched = if actual_port != port {
        Some(
            locale()
                .dev
                .port_switched
                .replace("{from}", &port.to_string())
                .replace("{to}", &actual_port.to_string()),
        )
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
            Ok(_) => println!("  ✓  {}", locale().dev.opening_browser),
            Err(e) => eprintln!("  ⚠  {}: {}", locale().dev.open_browser_failed, e),
        }
    }

    if watch {
        // Run the HTTP server as a background task; this task runs the watch loop.
        tokio::spawn(async move {
            axum::serve(listener, app).await.ok();
        });
        run_watch_loop(
            config,
            actual_port,
            &config.build.output_dir,
            &mut last_build_line,
        )
        .await?;
    } else {
        println!("  {}", locale().dev.press_ctrl_c_to_stop);
        axum::serve(listener, app).await?;
    }

    Ok(())
}

fn print_status_panel(
    port: u16,
    output_dir: &std::path::Path,
    last_build_line: Option<&str>,
    warning: Option<&str>,
) {
    let divider = panel_divider();
    println!("{}", divider);
    println!("  🌍  {}: http://localhost:{}", locale().dev.local, port);
    println!("  📁  {}: {}", locale().dev.serving, output_dir.display());
    if let Some(line) = last_build_line {
        println!("  🧱  {}", line);
    }
    if let Some(w) = warning {
        println!("  ⚠  {}", w);
    }
    println!("{}", divider);
    println!();
}

fn panel_divider() -> String {
    let width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(60);
    let len = width.max(24);
    "━".repeat(len)
}

fn format_last_build_line(ok: bool, elapsed: Duration, error_hint: Option<&str>) -> String {
    let status = if ok { "成功" } else { "失败" };
    let mut line = format!("{} | 用时 {:.1?}", status, elapsed);
    if let Some(hint) = error_hint {
        let display = if hint.len() > 50 { &hint[..50] } else { hint };
        line.push_str(&format!(" | {}", display));
    }
    line
}

fn format_building_line() -> String {
    locale().dev.build_rebuilding.clone()
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

#[derive(Debug, Clone, Copy)]
enum DevCmd {
    Rebuild,
    OpenBrowser,
    Clear,
}

/// Watches `src/`, `Cargo.toml`, `Tairitsu.toml`, and `public/` for changes and
/// triggers incremental rebuilds using the same pipeline as the initial build.
///
/// Debounces rapid saves (200 ms window) so a single `cargo save` operation does
/// not trigger multiple concurrent builds.
async fn run_watch_loop(
    config: &Config,
    port: u16,
    output_dir: &std::path::Path,
    last_build_line: &mut String,
) -> crate::Result<()> {
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

        println!("  ↻  {}", locale().dev.watching_for_changes);

        // Keep the watcher alive until the process exits.
        loop {
            std::thread::sleep(Duration::from_secs(3600));
        }
    });

    // ── Keyboard command listener ─────────────────────────────────────────────
    // Runs on a dedicated blocking thread so the tokio runtime is never stalled
    // waiting for stdin.  The user types a letter + Enter to send a command.
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<DevCmd>(8);
    tokio::task::spawn_blocking(move || {
        use std::io::BufRead;
        for line in std::io::stdin().lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(_) => continue,
            };
            let cmd = match line.trim() {
                "r" | "R" => Some(DevCmd::Rebuild),
                "o" | "O" => Some(DevCmd::OpenBrowser),
                "c" | "C" => Some(DevCmd::Clear),
                _ => None,
            };
            if let Some(c) = cmd {
                if cmd_tx.blocking_send(c).is_err() {
                    break;
                }
            }
        }
    });

    let debounce = tokio::time::Duration::from_millis(200);

    'watch: loop {
        // Wait for a rebuild trigger — either a file-system event or a keyboard
        // command.  `changed` is populated only for file-driven triggers.
        let mut changed: Vec<std::path::PathBuf> = Vec::new();

        let should_rebuild = tokio::select! {
            ev = rx.recv() => {
                let ev = match ev {
                    None => break 'watch,
                    Some(Err(e)) => {
                        eprintln!("  ⚠  {}: {}", locale().dev.watch_error, e);
                        continue;
                    }
                    Some(Ok(e)) => e,
                };
                if !matches!(
                    ev.kind,
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                ) {
                    continue;
                }

                // Collect paths for this event.
                changed = ev
                    .paths
                    .iter()
                    .filter_map(|p| p.strip_prefix(&project_root).ok().map(|r| r.to_path_buf()))
                    .collect();

                // Debounce: drain further events within the 200 ms window.
                let deadline = tokio::time::Instant::now() + debounce;
                while let Ok(Some(Ok(ev))) = tokio::time::timeout_at(deadline, rx.recv()).await {
                    changed.extend(ev.paths.iter().filter_map(|p| {
                        p.strip_prefix(&project_root).ok().map(|r| r.to_path_buf())
                    }));
                }
                changed.sort();
                changed.dedup();
                true
            }

            cmd = cmd_rx.recv() => {
                match cmd {
                    None => break 'watch,
                    Some(DevCmd::Rebuild) => {
                        println!("  ↻  {}", locale().dev.manual_rebuild_triggered);
                        true
                    }
                    Some(DevCmd::OpenBrowser) => {
                        let url = format!("http://localhost:{}", port);
                        println!("  ↗  {} {}", locale().dev.opening_url, url);
                        webbrowser::open(&url).ok();
                        false
                    }
                    Some(DevCmd::Clear) => {
                        // ANSI: clear screen and move cursor to top-left.
                        // indicatif redraws its persistent bars on the next tick.
                        print!("\x1b[2J\x1b[1;1H");
                        use std::io::Write;
                        std::io::stdout().flush().ok();
                        false
                    }
                }
            }

            _ = tokio::signal::ctrl_c() => {
                // Graceful shutdown on Ctrl+C
                println!();
                println!("  ✓  {}", locale().dev.stopping);
                std::process::exit(0);
            }
        };

        if !should_rebuild {
            continue;
        }

        if changed.is_empty() {
            println!("  ↻  {}", locale().dev.source_changed);
        } else if changed.len() == 1 {
            println!("  ↻  {}", changed[0].display());
        } else {
            println!("  ↻  {} {}", changed.len(), locale().dev.files_changed);
        }

        *last_build_line = format_building_line();
        print_status_panel(port, output_dir, Some(last_build_line), None);

        // Run the rebuild on a blocking thread so the tokio runtime stays alive.
        // The progress bar is managed internally by build_component.
        let rebuild_started = Instant::now();
        let config_clone = config.clone();
        let target = config.build.target.clone();
        let target_for_build = target.clone();
        let result = tokio::task::spawn_blocking(move || match target_for_build.as_str() {
            "component" => build_component(&config_clone, false, None),
            other => Err(crate::TairitsuPackagerError::BuildError(format!(
                "Unknown build target '{}'. Only 'component' is supported.",
                other
            ))),
        })
        .await
        .map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!("rebuild task panicked: {}", e))
        })?;

        let elapsed = rebuild_started.elapsed();
        match result {
            Ok(()) => {
                *last_build_line = format_last_build_line(true, elapsed, None);
                print_status_panel(port, output_dir, Some(last_build_line), None);
                println!(
                    "  ✓  {}  →  http://localhost:{}",
                    locale().dev.rebuilt,
                    port
                );
            }
            Err(e) => {
                let hint = extract_error_hint(e.to_string());
                *last_build_line = format_last_build_line(false, elapsed, Some(&hint));
                print_status_panel(port, output_dir, Some(last_build_line), None);
                eprintln!("  ✗  {}", e);
            }
        }
    }

    Ok(())
}

/// Extract a concise one-line description from a build error for the TUI status bar.
fn extract_error_hint(msg: String) -> String {
    // Prefer lines beginning with rustc-style "error[…]" or "error: …"
    for line in msg.lines() {
        let t = line.trim();
        if (t.starts_with("error[") || t.starts_with("error: "))
            && !t.contains("aborting due to")
            && t.len() > 6
        {
            return if t.len() > 55 {
                t[..55].to_string()
            } else {
                t.to_string()
            };
        }
    }
    // Fallback: first non-empty line, capped at 55 chars.
    let first = msg.lines().find(|l| !l.trim().is_empty()).unwrap_or(&msg);
    let first = first.trim();
    if first.len() > 55 {
        first[..55].to_string()
    } else {
        first.to_string()
    }
}
