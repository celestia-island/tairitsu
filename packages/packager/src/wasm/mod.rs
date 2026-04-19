use std::time::{Duration, Instant};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::{config::Config, daemon};

#[cfg(feature = "dev-server")]
fn locale() -> &'static crate::i18n::Translations {
    crate::i18n::translations()
}

fn find_workspace_root(manifest_dir: &std::path::Path) -> crate::Result<std::path::PathBuf> {
    let output = std::process::Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
            manifest_dir.join("Cargo.toml").to_str().unwrap(),
        ])
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

    // -- 1/5  check target -----------------------------------------------------
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

    // -- 2/5  compile ----------------------------------------------------------
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

    // -- 3/5  bundle assets ----------------------------------------------------
    pb.set_prefix("[3/5]");
    pb.set_message("bundle assets");
    let t = Instant::now();

    // Resolve output_dir relative to manifest_dir
    let output_dir = if config.build.output_dir.is_relative() {
        config.manifest_dir.join(&config.build.output_dir)
    } else {
        config.build.output_dir.clone()
    };

    let dest_wasm = output_dir.join(format!("{}.wasm", config.package.name.replace('-', "_")));

    tracing::info!("Creating output directory: {}", output_dir.display());
    std::fs::create_dir_all(&output_dir)?;

    tracing::info!(
        "Copying WASM from {} to {}",
        wasm_path.display(),
        dest_wasm.display()
    );
    std::fs::copy(&wasm_path, &dest_wasm)?;

    tracing::info!("Copying browser glue...");
    copy_browser_glue_with_output_dir(config, &output_dir, &pb)?;

    tracing::info!("Copying static public assets...");
    copy_static_public_assets_with_output_dir(config, &output_dir)?;

    tracing::info!("Compiling SCSS...");
    compile_project_scss_with_output_dir(config, &output_dir)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "bundle assets",
        t.elapsed()
    ));
    pb.inc(1);

    // -- 4/5  component wrapper ------------------------------------------------
    pb.set_prefix("[4/5]");
    pb.set_message("component wrapper");
    let t = Instant::now();
    prepare_component_wrapper_fallback(config, &dest_wasm, &output_dir, &pb)?;
    pb.println(format!(
        "     ✓  {:<28}  {:.1?}",
        "component wrapper",
        t.elapsed()
    ));
    pb.inc(1);

    // -- 5/5  HTML -------------------------------------------------------------
    pb.set_prefix("[5/5]");
    pb.set_message("generate HTML");
    let t = Instant::now();
    generate_component_html_with_output_dir(config, &output_dir)?;
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

    // Log successful build if in daemon mode
    if daemon::is_daemon() {
        let _ = daemon::append_build_log("component", true, None);
    }

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
        "--manifest-path",
        config.manifest_dir.join("Cargo.toml").to_str().unwrap(),
        "--message-format=json-diagnostic-rendered-ansi",
    ]);
    if release {
        cmd.arg("--release");
    } else {
        cmd.args(["--profile", "dev-wasm"]);
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
        if let Some(after_hash) = package_id.split('#').nth(1)
            && let Some(name) = after_hash.split('@').next()
        {
            return name;
        }
        // Fallback: return the whole thing
        package_id
    }

    // Thread: parse JSON messages from stdout and update the progress bar
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stdout).lines() {
            let Ok(line) = line else { continue };
            // Try to parse as cargo JSON message
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line)
                && let Some(reason) = msg.get("reason").and_then(|r| r.as_str())
            {
                match reason {
                    "compiler-artifact" => {
                        // A crate is being compiled
                        if let Some(package_id) = msg.get("package_id").and_then(|p| p.as_str()) {
                            let crate_name = extract_crate_name(package_id);
                            // Only update for lib crates (not build scripts)
                            if let Some(target) = msg.get("target")
                                && target
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
                        if let Some(package_id) = msg.get("package_id").and_then(|p| p.as_str()) {
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
    });

    let status = child.wait()?;
    // Don't finish/clear here - the caller manages the progress bar lifecycle

    if !status.success() {
        // Log failed build if in daemon mode
        if daemon::is_daemon() {
            let _ = daemon::append_build_log(
                "component",
                false,
                Some("cargo build --target wasm32-wasip2 failed"),
            );
        }
        return Err(crate::TairitsuPackagerError::BuildError(
            "cargo build --target wasm32-wasip2 failed".to_string(),
        ));
    }

    let workspace_root = find_workspace_root(&config.manifest_dir)?;
    let profile = if release { "release" } else { "dev-wasm" };
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

mod browser_glue_bundle {
    include!(concat!(env!("OUT_DIR"), "/browser_glue_bundle.rs"));
}

fn write_browser_glue_bundle(config: &Config, output_dir: &std::path::Path) -> crate::Result<()> {
    let glue_path = config.build.browser_glue_path.trim_start_matches('/');
    let bundle_dest = output_dir.join(glue_path);

    if let Some(parent) = bundle_dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(&bundle_dest, browser_glue_bundle::BROWSER_GLUE_BUNDLE)?;

    Ok(())
}

fn resolve_import_to_modules(
    full_import: &str,
    symbols_str: &str,
    sym_map: &std::collections::HashMap<String, String>,
    single_quote: bool,
) -> String {
    let q = if single_quote { '\'' } else { '"' };

    let symbols: Vec<&str> = symbols_str.split(',').map(|s| s.trim()).collect();

    let mut module_imports: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for sym in &symbols {
        let clean = sym.split_whitespace().next().unwrap_or(sym);
        if clean.is_empty() {
            continue;
        }
        // Exact match first
        if let Some(mod_name) = sym_map.get(clean) {
            module_imports
                .entry(mod_name.clone())
                .or_default()
                .push(clean.to_string());
        } else if let Some((alias_mod, _)) = sym_map
            .iter()
            .find(|(k, _)| {
                let clean_str: &str = clean;
                k.ends_with(clean_str) && k.len() > clean_str.len()
            })
        {
            module_imports
                .entry(alias_mod.clone())
                .or_default()
                .push(clean.to_string());
        } else {
            tracing::warn!(
                "Glue symbol '{}' not found in any module (map has {} symbols)",
                clean,
                sym_map.len()
            );
        }
    }

    if module_imports.is_empty() {
        return String::new();
    }

    let mut lines = Vec::new();
    for (mod_name, syms) in &module_imports {
        lines.push(format!(
            "import {{ {} }} from {}../browser-glue/glue/{}.js{}",
            syms.join(", "),
            q,
            mod_name,
            q
        ));
    }
    lines.join("\n  ")
}

fn flatten_barrel_exports(barrel_path: &std::path::Path) -> crate::Result<()> {
    let content = std::fs::read_to_string(barrel_path).map_err(|e| {
        crate::TairitsuPackagerError::BuildError(format!(
            "Failed to read barrel {}: {}",
            barrel_path.display(), e
        ))
    })?;

    let mut needs_flatten = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("export * as ") || (trimmed.contains("export * from") && trimmed.contains("glue/index")) {
            needs_flatten = true;
            break;
        }
    }

    if !needs_flatten {
        return Ok(());
    }

    let glue_dir = barrel_path.parent().map(|p| p.join("glue"));
    let Some(glue_dir) = glue_dir else { return Ok(()); };

    let sym_map = build_symbol_module_map(&glue_dir);

    let mut module_syms: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    for (sym, mod_name) in &sym_map {
        module_syms
            .entry(mod_name.clone())
            .or_default()
            .push(sym.clone());
    }

    let mut lines = vec![
        "/**".to_string(),
        " * @tairitsu/browser-glue — flattened barrel (auto-generated)".to_string(),
        " * Combines exports from all glue submodules into named re-exports.".to_string(),
        " */".to_string(),
    ];
    for (mod_name, syms) in &module_syms {
        if syms.is_empty() {
            continue;
        }
        lines.push(format!(
            "export {{ {} }} from \"./glue/{}.js\";",
            syms.join(", "),
            mod_name
        ));
    }

    std::fs::write(barrel_path, lines.join("\n")).map_err(|e| {
        crate::TairitsuPackagerError::BuildError(format!(
            "Failed to write flattened barrel {}: {}",
            barrel_path.display(), e
        ))
    })?;

    tracing::info!(
        "Flattened barrel {} with {} modules ({} total symbols)",
        barrel_path.display(),
        module_syms.len(),
        sym_map.len()
    );
    Ok(())
}

fn build_symbol_module_map(
    glue_dir: &std::path::Path,
) -> std::collections::HashMap<String, String> {
    let mut map: std::collections::HashMap<String, (String, bool)> = std::collections::HashMap::new();
    let Ok(entries) = std::fs::read_dir(glue_dir) else {
        return std::collections::HashMap::new()
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.extension().map_or(false, |e| e == "js") {
            continue;
        }
        let Some(module_name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        for line in content.lines() {
            let trimmed = line.trim();
            let export_start = trimmed
                .find("export ")
                .or_else(|| trimmed.find("export\t"))
                .unwrap_or(usize::MAX);
            if export_start == usize::MAX {
                continue;
            }
            let after_export = &trimmed[export_start + 7..];
            let prefix = if after_export.starts_with("function ") {
                "function "
            } else if after_export.starts_with("const ") {
                "const "
            } else if after_export.starts_with("let ") {
                "let "
            } else if after_export.starts_with("var ") {
                "var "
            } else if after_export.starts_with("async function ") {
                "async function "
            } else {
                continue;
            };
            if let Some(rest) = after_export.strip_prefix(prefix) {
                if let Some(sym_full) = rest.split_whitespace().next() {
                    let sym_name = sym_full
                        .split('(')
                        .next()
                        .unwrap_or(sym_full)
                        .trim()
                        .to_string();
                    if !sym_name.is_empty() {
                        // Check if first param is 'self' — prefer non-self versions
                        let has_self = rest.contains("(self)") || rest.contains("(self,");
                        match map.get(&sym_name) {
                            Some((_other_mod, other_has_self)) => {
                                // Prefer non-self over self-parameterized versions
                                if *other_has_self && !has_self {
                                    map.insert(sym_name, (module_name.to_string(), has_self));
                                }
                            }
                            None => {
                                map.insert(sym_name, (module_name.to_string(), has_self));
                            }
                        }
                    }
                }
            }
        }
    }
    map.into_iter().map(|(k, (v, _))| (k, v)).collect()
}

fn rewrite_glue_import_line(
    import_line: &str,
    sym_map: &std::collections::HashMap<String, String>,
    single_quote: bool,
) -> String {
    let re_symbols = regex::Regex::new(r"\{([^}]+)\}").unwrap();
    let re_source = if single_quote {
        regex::Regex::new(r"'@tairitsu-glue/[^']*'").unwrap()
    } else {
        regex::Regex::new(r#""@tairitsu-glue/[^"]*""#).unwrap()
    };

    let symbols_str = re_symbols
        .captures(import_line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("");

    let symbols: Vec<&str> = symbols_str.split(',').map(|s| s.trim()).collect();

    let mut module_imports: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();
    let mut unknown = Vec::new();

    for sym in &symbols {
        let clean = sym.split_whitespace().next().unwrap_or(sym);
        if clean.is_empty() {
            continue;
        }
        if let Some(mod_name) = sym_map.get(clean) {
            module_imports
                .entry(mod_name.clone())
                .or_default()
                .push(clean.to_string());
        } else if let Some((alias_mod, _)) = sym_map
            .iter()
            .find(|(k, _)| {
                let clean_str: &str = clean;
                k.ends_with(clean_str) && k.len() > clean_str.len()
            })
        {
            module_imports
                .entry(alias_mod.clone())
                .or_default()
                .push(clean.to_string());
        } else {
            unknown.push(clean.to_string());
        }
    }

    let q = if single_quote { "'" } else { "\"" };
    let mut lines = Vec::new();
    for (mod_name, syms) in &module_imports {
        if syms.is_empty() {
            continue;
        }
        lines.push(format!(
            "import {{ {} }} from {}../browser-glue/glue/{}.js{}",
            syms.join(", "), q, mod_name, q
        ));
    }
    // Drop unknown symbols silently — they're likely unimplemented WIT stubs
    // that would cause module load failures if imported.
    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n  ")
    }
}

fn copy_browser_glue_with_output_dir(
    config: &Config,
    output_dir: &std::path::Path,
    _pb: &ProgressBar,
) -> crate::Result<()> {
    write_browser_glue_bundle(config, output_dir)?;

    // Also copy the full browser-glue dist so that jco-generated wrapper
    // ES module imports (e.g. `import { setProperty } from '@tairitsu/browser-glue'`)
    // can resolve. The dist contains index.js (main barrel) + glue/*.js submodules.
    // Try tairitsu workspace first (where browser-glue is built), then local.
    let glue_dist_candidates: &[&str] = &[
        "../../../tairitsu/packages/browser-glue/dist", // hikari/examples/website → tairitsu
        "../../packages/browser-glue/dist",          // tairitsu/examples/website → tairitsu
        "../packages/browser-glue/dist",             // any project examples/* → workspace root
        "packages/browser-glue/dist",               // workspace root itself
    ];
    let mut copied = false;
    for candidate in glue_dist_candidates {
        let glue_dist = config.manifest_dir.join(candidate);
        if glue_dist.exists() {
            let dest_glue = output_dir.join("browser-glue");
            std::fs::create_dir_all(&dest_glue)?;
            for entry in walkdir::WalkDir::new(&glue_dist) {
                let entry = entry.map_err(|e| {
                    crate::TairitsuPackagerError::BuildError(format!("Failed to walk browser-glue dist: {}", e))
                })?;
                let path = entry.path();
                let rel = path.strip_prefix(&glue_dist).map_err(|_| {
                    crate::TairitsuPackagerError::BuildError("Failed to compute relative path".to_string())
                })?;
                let dest = dest_glue.join(rel);
                if path.is_dir() {
                    std::fs::create_dir_all(&dest)?;
                } else {
                    std::fs::copy(path, &dest)?;
                }
            }
            copied = true;

            let top_barrel = dest_glue.join("index.js");
            if top_barrel.exists() {
                if let Err(e) = flatten_barrel_exports(&top_barrel) {
                    tracing::warn!("Failed to flatten top-level barrel: {}", e);
                }
            }
            break;
        }
    }
    if !copied {
        tracing::warn!("browser-glue dist not found — jco wrapper imports may fail at runtime");
    }

    Ok(())
}

fn copy_static_public_assets_with_output_dir(
    config: &Config,
    output_dir: &std::path::Path,
) -> crate::Result<()> {
    let public_dir = config.manifest_dir.join("public");

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
        let dest = output_dir.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(path, dest)?;
    }

    Ok(())
}

fn compile_project_scss_with_output_dir(
    config: &Config,
    output_dir: &std::path::Path,
) -> crate::Result<()> {
    let project_root = &config.manifest_dir;

    tracing::info!("SCSS project root: {}", project_root.display());

    // Use new SCSS configuration system
    let results = crate::styles::compile_scss_with_config(&config.scss, project_root, output_dir)
        .map_err(|e| {
        crate::TairitsuPackagerError::BuildError(format!("Failed to compile SCSS: {}", e))
    })?;

    // Log compiled files
    for result in results {
        tracing::info!("Compiled SCSS: {}", result.output_path.display());
    }

    Ok(())
}

fn prepare_component_wrapper_fallback(
    config: &Config,
    component_wasm_path: &std::path::Path,
    output_dir: &std::path::Path,
    pb: &ProgressBar,
) -> crate::Result<()> {
    write_component_wrapper_loader(config, component_wasm_path, output_dir)?;

    if !try_generate_component_wrapper(config, component_wasm_path, output_dir, pb)? {
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
            output_dir.join("component-wrapper").display()
        ));
    }

    // Always normalize wrapper imports if wrapper files already exist.
    rewrite_wrapper_imports_to_esm(output_dir)?;

    Ok(())
}

fn rewrite_wrapper_imports_to_esm(output_dir: &std::path::Path) -> crate::Result<()> {
    let wrapper_dir = output_dir.join("component-wrapper");
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
    _config: &Config,
    component_wasm_path: &std::path::Path,
    output_dir: &std::path::Path,
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
        "/src/wasm/component-wrapper-loader.template.js"
    ))
    .replace("__WASM_STEM__", wasm_stem);

    let loader_path = output_dir.join("component-wrapper-loader.js");
    std::fs::write(loader_path, loader)?;
    Ok(())
}

fn try_generate_component_wrapper(
    _config: &Config,
    component_wasm_path: &std::path::Path,
    output_dir: &std::path::Path,
    pb: &ProgressBar,
) -> crate::Result<bool> {
    let wrapper_dir = output_dir.join("component-wrapper");
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
        if named.exists() { Some(named) } else { None }
    };

    if let Some(main) = wrapper_main {
        let wrapper_mtime = std::fs::metadata(&main)?.modified().ok();
        let wasm_mtime = std::fs::metadata(component_wasm_path)?.modified().ok();
        if let (Some(w), Some(c)) = (wrapper_mtime, wasm_mtime)
            && w >= c
        {
            return Ok(true);
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
                    // Post-process the generated JS to replace tairitsu-browser:full/ imports
                    // with @tairitsu-glue/ imports which are valid bare module specifiers
                    let js_files = vec![
                        wrapper_dir.join("index.js"),
                        wrapper_dir.join(format!("{}.js", wasm_stem)),
                    ];
                    for js_file in js_files {
                        if js_file.exists()
                            && let Ok(mut content) = std::fs::read_to_string(&js_file)
                        {
                            let original = content.clone();
                            // Step 1: WIT world names → @tairitsu-glue intermediate
                            content = content
                                .replace("from 'tairitsu-browser:full/", "from '@tairitsu-glue/");
                            content = content
                                .replace("from \"tairitsu-browser:full/", "from \"@tairitsu-glue/");

                            // Step 2: All @tairitsu-glue/ imports now route through import map
                            // (__tairitsu_glue__.js provides all needed interfaces with shared handles)
                            let _import_map_interfaces: std::collections::HashSet<&str> =
                                [].into_iter().collect();

                            let re_import = regex::Regex::new(
                                r"(?:import\s*\{([^}]+)\}\s*from\s*'@tairitsu-glue/([^']*)'\s*;?)",
                            )
                            .unwrap();
                            let re_import_d = regex::Regex::new(
                                r#"(?:import\s*\{([^}]+)\}\s*from\s*"@tairitsu-glue/([^"]*)"\s*;?)"#,
                            )
                            .unwrap();

                            content = re_import
                                .replace_all(&content, |caps: &regex::Captures| {
                                    let m0 = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                                    let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                                    let m2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                                    if m1.is_empty() {
                                        return String::new();
                                    }
                                    let iface_name = m2.split('/').next().unwrap_or("");

                                    // Split symbols: import-map-available vs needs-glue-fallback
                                    let mut map_syms = Vec::new();
                                    for sym in m1.trim().split(',') {
                                        let s = sym.trim();
                                        if s.is_empty() { continue; }
                                        // All interfaces now provided by __tairitsu_glue__.js import map
                                        map_syms.push(s.to_string());
                                    }

                                    let mut parts = Vec::new();
                                    if !map_syms.is_empty() {
                                        parts.push(format!(
                                            "import {{ {} }} from '@tairitsu-glue/{}';",
                                            map_syms.join(", "), m2
                                        ));
                                    }
                                        // Resolve each symbol to its own module
                                    parts.join("\n  ")
                                })
                                .to_string();
                            content = re_import_d
                                .replace_all(&content, |caps: &regex::Captures| {
                                    let m0 = caps.get(0).map(|m| m.as_str()).unwrap_or("");
                                    let m1 = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                                    let m2 = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                                    if m1.is_empty() {
                                        return String::new();
                                    }
                                    let iface_name = m2.split('/').next().unwrap_or("");

                                    let mut map_syms = Vec::new();
                                    for sym in m1.trim().split(',') {
                                        let s = sym.trim();
                                        if s.is_empty() { continue; }
                                        map_syms.push(s.to_string());
                                    }

                                    let mut parts = Vec::new();
                                    if !map_syms.is_empty() {
                                        parts.push(format!(
                                            "import {{ {} }} from \"@tairitsu-glue/{}\";",
                                            map_syms.join(", "), m2
                                        ));
                                    }
                                    parts.join("\n  ")
                                })
                                .to_string();

                            if content != original {
                                std::fs::write(&js_file, content)?;
                            }
                        }

                        // Patch preview2-shim resource instanceof checks that fail
                        // due to cross-module class identity issues (jco + preview2-shim).
                        // The getStdout/getStderr/subscribe calls return valid objects but
                        // from a different module instance than the expected class.
                        if let Ok(mut content) = std::fs::read_to_string(&js_file) {
                            let original = content.clone();
                            let patterns = [
                                ("OutputStream", "\"OutputStream\""),
                                ("InputStream", "\"InputStream\""),
                                ("Pollable", "\"Pollable\""),
                                ("TerminalOutput", "\"TerminalOutput\""),
                                ("TerminalInput", "\"TerminalInput\""),
                            ];
                            for (class_name, label) in patterns {
                                let check = format!(
                                    "if (!(ret instanceof {})) {{\n      throw new TypeError('Resource error: Not a valid {} resource.');\n    }}",
                                    class_name, label
                                );
                                let catch_check = format!(
                                    "if (!(e instanceof {})) {{\n      throw new TypeError('Resource error: Not a valid {} resource.');\n    }}",
                                    class_name, label
                                );
                                let comment = format!("/* {} instanceof check patched for browser compat */", class_name);
                                content = content.replace(&check, &comment);
                                content = content.replace(&catch_check, &comment);
                            }
                            if content != original {
                                std::fs::write(&js_file, content)?;
                            }
                        }
                    }
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

fn generate_component_html_with_output_dir(
    config: &Config,
    output_dir: &std::path::Path,
) -> crate::Result<()> {
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

    // Browser glue bundle path (relative to output directory)
    let glue_bundle_path = &config.build.browser_glue_path;

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
<body class="{body_class}" data-booting>
    <style>[data-booting] {{ visibility: hidden !important; }}</style>
    <div id="app">Loading...</div>
    <!-- Import map (WASI preview2-shim + tairitsu-glue interfaces) is created
         dynamically by __tairitsu_glue__.js, which runs synchronously before
         any module scripts, guaranteeing a single complete import map. -->
    <script src="{glue_bundle_path}?v={v}"></script>
    <script type="module">
        import {{ instantiateWithWrapper }} from '/component-wrapper-loader.js?v={v}';

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

        // Build imports object from registered browser glue interfaces
        const buildImports = () => {{
            const imports = {{}};
            if (globalThis.__TAIRITSU_GLUE__ && globalThis.__TAIRITSU_GLUE__.INTERFACES) {{
                for (const [shortName, exports] of Object.entries(globalThis.__TAIRITSU_GLUE__.INTERFACES)) {{
                    const ifaceName = shortName.replace('@tairitsu-glue/', '');
                    const fullName = `tairitsu-browser:full/${{ifaceName}}@0.2.0`;
                    imports[fullName] = exports;
                }}
            }}
            return imports;
        }};

        let bootInvoked = false;

        // Strategy: always try the jco component wrapper first (avoids a
        // redundant 50MB fetch of the raw WASM just to inspect magic bytes).
        // The wrapper internally fetches only the core module it needs.
        // Fall back to direct WASM loading only if no wrapper is available.
        try {{
            const wrapperResult = await instantiateWithWrapper(buildImports());
            if (globalThis.__setWasmExports && wrapperResult) {{
                globalThis.__setWasmExports(wrapperResult);
            }}
            bootInvoked = await tryInvokeBootExports(wrapperResult);
        }} catch (wrapperErr) {{
            console.error('[tairitsu] Component wrapper failed, falling back to direct WASM load:', wrapperErr);
            const response = await fetch('/{wasm_file}');
            const bytes = await response.arrayBuffer();
            const magic = new Uint8Array(bytes, 0, 8);
            const isWasm =
                magic[0] === 0x00 && magic[1] === 0x61 && magic[2] === 0x73 && magic[3] === 0x6d;
            const isComponent = isWasm &&
                magic[4] === 0x0d && magic[5] === 0x00 && magic[6] === 0x01 && magic[7] === 0x00;

            if (isComponent && typeof WebAssembly.Component === 'function') {{
                const component = new WebAssembly.Component(bytes);
                const componentResult = await WebAssembly.instantiate(component, buildImports());
                if (globalThis.__setWasmExports && componentResult) {{
                    globalThis.__setWasmExports(componentResult);
                }}
                bootInvoked = await tryInvokeBootExports(componentResult);
            }} else if (isComponent) {{
                console.error('[tairitsu] WASM Component detected but WebAssembly.Component is not supported in this browser. The jco-transpiled wrapper must be used.');
                setAppStatus('Failed to load: WASM Components require the jco wrapper, which failed to load. Check console for details.');
            }} else {{
                const module = await WebAssembly.compile(bytes);
                const moduleResult = await WebAssembly.instantiate(module, buildImports());
                if (globalThis.__setWasmExports && moduleResult) {{
                    globalThis.__setWasmExports(moduleResult);
                }}
                bootInvoked = await tryInvokeBootExports(moduleResult);
            }}
        }}

        if (!bootInvoked) {{
            setAppStatus('Component initialized (no exported run/start entry found).');
        }} else {{
            clearLoadingIfUnchanged();
        }}

        // Wait for all stylesheets to finish loading before running any
        // post-boot DOM fixups.  CSS <link>s in <head> block rendering but
        // a deferred module script can execute before every stylesheet has
        // fully parsed (edge-case on slow networks / large bundles).
        const stylesReady = () => {{
            if (!document.styleSheets) return Promise.resolve();
            const pending = [];
            for (const sheet of document.styleSheets) {{
                try {{ void sheet.cssRules; }} catch (_) {{
                    // cssRules throws when the stylesheet is still loading
                    // (cross-origin sheets always throw, but those are fine
                    // since they are not our component styles).
                    pending.push(sheet);
                }}
            }}
            if (pending.length === 0) return Promise.resolve();
            return new Promise(resolve => {{
                let count = 0;
                const check = () => {{
                    count++;
                    if (count > 50) {{ resolve(); return; }}
                    let stillPending = 0;
                    for (const s of pending) {{
                        try {{ void s.cssRules; }} catch (_) {{ stillPending++; }}
                    }}
                    if (stillPending === 0) resolve();
                    else setTimeout(check, 40);
                }};
                check();
            }});
        }};

        // Fix SVG elements created with wrong namespace (HTML instead of SVG).
        // WIT document::createElement always creates HTML-namespaced elements,
        // which makes SVG graphics invisible. This post-process step replaces
        // any HTML-namespaced <svg> elements with proper SVG namespaced ones.
        const fixSvgNamespaces = () => {{
            const SVG_NS = 'http://www.w3.org/2000/svg';
            document.querySelectorAll('svg').forEach(svg => {{
                if (svg.namespaceURI === SVG_NS) return;
                const parent = svg.parentNode;
                if (!parent) return;
                const newSvg = document.createElementNS(SVG_NS, 'svg');
                for (const attr of svg.attributes) {{
                    if (attr.name.toLowerCase() === 'viewbox') {{
                        newSvg.setAttribute('viewBox', attr.value);
                    }} else {{
                        newSvg.setAttribute(attr.name, attr.value);
                    }}
                }}
                newSvg.innerHTML = svg.innerHTML;
                parent.replaceChild(newSvg, svg);
            }});
        }};

        // WASM just replaced #app content; re-run SPA router so the
        // correct page is activated for the current URL path.
        // The WASM boot function may spawn async DOM work (rAF, setTimeout,
        // etc.) that finishes *after* the await resolves.  A single rAF
        // is not enough — we wait for DOM mutations inside #app to settle
        // before calling navigate(), so every .hikari-page element exists.
        const waitForDomSettle = (root, maxWait) => {{
            return new Promise(resolve => {{
                const deadline = Date.now() + maxWait;
                let timer = null;
                let settled = false;

                const tryResolve = () => {{
                    if (settled) return;
                    settled = true;
                    if (timer) {{ clearTimeout(timer); timer = null; }}
                    observer.disconnect();
                    resolve();
                }};

                const observer = new MutationObserver(() => {{
                    // mutations happening — reset the settle timer
                    if (timer) clearTimeout(timer);
                    if (Date.now() >= deadline) {{ tryResolve(); return; }}
                    timer = setTimeout(tryResolve, 80);
                }});

                observer.observe(root, {{
                    childList: true,
                    subtree: true,
                    attributes: false,
                }});

                // If no mutations fire within a short window, resolve.
                timer = setTimeout(tryResolve, 120);

                // Absolute safety net.
                setTimeout(tryResolve, maxWait);
            }});
        }};

        (async () => {{
            await stylesReady();

            const appRoot = document.getElementById('app');
            if (appRoot) {{
                await waitForDomSettle(appRoot, 2000);
            }}

            fixSvgNamespaces();

            if (typeof navigate === 'function') {{
                navigate();
            }} else {{
                window.dispatchEvent(new PopStateEvent('popstate'));
            }}

            document.body.removeAttribute('data-booting');
        }})();

        // Glow mouse-following spotlight effect
        (function initGlow() {{
            let rafId = null;
            const activeGlow = new Map();

            document.addEventListener('mouseenter', function(e) {{
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow || activeGlow.has(glow)) return;
                activeGlow.set(glow, {{ x: 50, y: 50 }});
                if (!rafId) scheduleFrame();
            }}, true);

            document.addEventListener('mousemove', function(e) {{
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow || !activeGlow.has(glow)) return;
                const rect = glow.getBoundingClientRect();
                if (rect.width > 0 && rect.height > 0) {{
                    activeGlow.get(glow).x = ((e.clientX - rect.left) / rect.width * 100).toFixed(1);
                    activeGlow.get(glow).y = ((e.clientY - rect.top) / rect.height * 100).toFixed(1);
                }}
            }}, true);

            document.addEventListener('mouseleave', function(e) {{
                const glow = e.target.closest ? e.target.closest('[data-glow]') : null;
                if (!glow) return;
                activeGlow.delete(glow);
                glow.style.setProperty('--glow-x', '50%');
                glow.style.setProperty('--glow-y', '50%');
            }}, true);

            function scheduleFrame() {{
                rafId = requestAnimationFrame(function tick() {{
                    for (const [el, pos] of activeGlow) {{
                        el.style.setProperty('--glow-x', pos.x + '%');
                        el.style.setProperty('--glow-y', pos.y + '%');
                    }}
                    if (activeGlow.size > 0) {{
                        rafId = requestAnimationFrame(tick);
                    }} else {{
                        rafId = null;
                    }}
                }});
            }}
        }})();

        // Hot-reload client: connects to dev server SSE endpoint.
        // On rebuild notification, reloads the page. Gives up after 3
        // consecutive failures and suggests using release builds.
        (function initHotReload() {{
            let failures = 0;
            const MAX_FAILURES = 3;
            function connect() {{
                const es = new EventSource('/__tairitsu_reload');
                es.onmessage = function(ev) {{
                    if (ev.data === 'reload') {{
                        console.log('[tairitsu] rebuild detected, reloading…');
                        location.reload();
                    }}
                }};
                es.onerror = function() {{
                    failures++;
                    es.close();
                    if (failures >= MAX_FAILURES) {{
                        console.warn(
                            '[tairitsu] hot-reload connection failed ' + MAX_FAILURES +
                            ' times. Dev server may not be running.' +
                            '\\nFor production builds, use: tairitsu build --release'
                        );
                        return;
                    }}
                    setTimeout(connect, 3000);
                }};
            }}
            connect();
        }})();
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

    let html_path = output_dir.join("index.html");
    std::fs::write(&html_path, &html)?;

    generate_route_html_files(config, output_dir, &html)?;

    Ok(())
}

fn generate_route_html_files(
    config: &Config,
    output_dir: &std::path::Path,
    base_html: &str,
) -> crate::Result<()> {
    use crate::config::DiscoveredRoute;

    let routes = config.discovered_routes();
    if routes.is_empty() {
        return Ok(());
    }

    let non_root_routes: Vec<&DiscoveredRoute> = routes
        .iter()
        .filter(|r| !r.is_root())
        .collect();

    if non_root_routes.is_empty() {
        return Ok(());
    }

    let route_preload_script = |route_path: &str| -> String {
        format!(
            "<script>if(history.replaceState){{history.replaceState(null,null,{path})}}</script>",
            path = serde_json::to_string(route_path).unwrap_or_else(|_| format!("'{}'", route_path))
        )
    };

    for route in &non_root_routes {
        let fs_route = route.fs_path();
        let route_dir = output_dir.join(fs_route);

        std::fs::create_dir_all(&route_dir).map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!(
                "Failed to create route directory {}: {}",
                route_dir.display(),
                e
            ))
        })?;

        let preload = route_preload_script(&route.path);
        let route_html = base_html.replace(
            "<div id=\"app\">Loading...</div>",
            &format!("<div id=\"app\">Loading...</div>\n    {}", preload),
        );

        let route_file = route_dir.join("index.html");
        std::fs::write(&route_file, &route_html).map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!(
                "Failed to write route HTML {}: {}",
                route_file.display(),
                e
            ))
        })?;
    }

    tracing::info!(
        "Generated {} route HTML files for direct URL access",
        non_root_routes.len()
    );

    Ok(())
}

/// Middleware that prevents browsers from caching JS/WASM/HTML assets in dev mode.
/// Without this, browsers serve stale cached modules even after a file changes.
#[cfg(feature = "dev-server")]
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

#[cfg(feature = "dev-server")]
pub async fn dev_server(config: &Config, port: u16, open: bool, watch: bool) -> crate::Result<()> {
    use axum::{Router, middleware, response::Html, routing::get};
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

    // Broadcast channel for hot-reload SSE notifications.
    // When a rebuild finishes, the watch loop sends () here;
    // the browser client receives it and re-fetches WASM.
    let (reload_tx, _) = tokio::sync::broadcast::channel::<()>(8);

    // Initial build (no MultiProgress — let cargo write directly to terminal).
    let initial_started = Instant::now();
    match config.build.target.as_str() {
        "component" => build_component(config, false, None).inspect_err(|e| {
            if crate::daemon::is_daemon() {
                let _ = crate::daemon::signal_failed(&e.to_string());
            }
        })?,
        other => {
            let err = crate::TairitsuPackagerError::BuildError(format!(
                "Unknown build target '{}'. Only 'component' is supported.",
                other
            ));
            if crate::daemon::is_daemon() {
                let _ = crate::daemon::signal_failed(&err.to_string());
            }
            return Err(err);
        }
    }
    let initial_elapsed = initial_started.elapsed();

    if crate::daemon::is_daemon() {
        let _ = crate::daemon::signal_ready();
        #[cfg(unix)]
        crate::daemon::daemonize_self().map_err(|e| {
            crate::TairitsuPackagerError::BuildError(format!("daemonize failed: {}", e))
        })?;
        println!("  ✓  Initial build succeeded — daemonizing...");
    }

    let dist_dir = config.build.output_dir.clone();

    // Always read index.html from disk on each request so watch-mode rebuilds
    // are served immediately without needing a server restart.
    let dist_for_index = dist_dir.clone();
    let pkg_name = config.package.name.clone();
    // Clone dist_dir for the SPA fallback handler
    let dist_for_spa = dist_for_index.clone();
    let spa_pkg_name = pkg_name.clone();

    let discovered = config.discovered_routes();
    let known_route_paths: Vec<String> = discovered
        .iter()
        .filter(|r| !r.is_root())
        .map(|r| r.path.clone())
        .collect();

    let spa_fallback = Router::new().fallback(get(move |req: axum::extract::Request| {
        let dist = dist_for_spa.clone();
        let pkg = spa_pkg_name.clone();
        let routes = known_route_paths.clone();
        async move {
            let path = req.uri().path().trim_end_matches('/');

            let route_file = if !path.is_empty() && routes.iter().any(|r| r == path || format!("{}/", r) == *path) {
                let clean_path = path.strip_prefix('/').unwrap_or(path);
                Some(dist.join(clean_path).join("index.html"))
            } else {
                None
            };

            let content = match route_file {
                Some(ref fp) if fp.exists() => std::fs::read_to_string(fp),
                _ => Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "no route file",
                )),
            }
            .or_else(|_| {
                std::fs::read_to_string(dist.join("index.html"))
            })
            .unwrap_or_else(|_| {
                format!(
                    "<!DOCTYPE html><html><head><title>{}</title></head>\
                         <body><div id=\"app\">Loading…</div></body></html>",
                    pkg
                )
            });
            Html(content)
        }
    }));
    let reload_tx_for_route = reload_tx.clone();
    let app = Router::new()
        .route("/__tairitsu_reload", get(move || {
            let tx = reload_tx_for_route.clone();
            async move {
                use axum::response::sse::{Event, Sse};
                use axum::response::IntoResponse;
                use futures::stream::StreamExt;
                let receiver = tx.subscribe();
                let stream = tokio_stream::wrappers::BroadcastStream::new(receiver).map(|result| {
                    match result {
                        Ok(()) => Ok::<_, std::io::Error>(Event::default().data("reload")),
                        Err(_) => Err(std::io::Error::new(std::io::ErrorKind::Other, "lagged")),
                    }
                });
                Sse::new(stream).keep_alive(
                    axum::response::sse::KeepAlive::new()
                        .interval(Duration::from_secs(15))
                        .text("ping"),
                ).into_response()
            }
        }))
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
        .fallback_service(ServeDir::new(dist_dir).fallback(spa_fallback))
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
            reload_tx,
        )
        .await?;
    } else {
        println!("  {}", locale().dev.press_ctrl_c_to_stop);
        axum::serve(listener, app).await?;
    }

    Ok(())
}

#[cfg(feature = "dev-server")]
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

#[cfg(feature = "dev-server")]
fn panel_divider() -> String {
    let width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(60);
    let len = width.max(24);
    "━".repeat(len)
}

#[cfg(feature = "dev-server")]
fn format_last_build_line(ok: bool, elapsed: Duration, error_hint: Option<&str>) -> String {
    let status = if ok { "成功" } else { "失败" };
    let mut line = format!("{} | 用时 {:.1?}", status, elapsed);
    if let Some(hint) = error_hint {
        let display = if hint.len() > 50 { &hint[..50] } else { hint };
        line.push_str(&format!(" | {}", display));
    }
    line
}

#[cfg(feature = "dev-server")]
fn format_building_line() -> String {
    locale().dev.build_rebuilding.clone()
}
/// Try binding localhost starting from `preferred_port` and automatically
/// fallback to higher ports when the preferred one is already occupied.
#[cfg(feature = "dev-server")]
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

#[cfg(feature = "dev-server")]
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
#[cfg(feature = "dev-server")]
async fn run_watch_loop(
    config: &Config,
    port: u16,
    output_dir: &std::path::Path,
    last_build_line: &mut String,
    reload_tx: tokio::sync::broadcast::Sender<()>,
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

    // -- Keyboard command listener ---------------------------------------------
    // Runs on a dedicated blocking thread so the tokio runtime is never stalled
    // waiting for stdin.  The user types a letter + Enter to send a command.
    // Skipped in daemon mode where stdin is unavailable.
    let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<DevCmd>(8);
    if !crate::daemon::is_daemon() {
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
                if let Some(c) = cmd
                    && cmd_tx.blocking_send(c).is_err()
                {
                    break;
                }
            }
        });
    }

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
                    .filter_map(|p: &std::path::PathBuf| p.strip_prefix(&project_root).ok().map(|r: &std::path::Path| r.to_path_buf()))
                    .collect();

                // Debounce: drain further events within the 200 ms window.
                let deadline = tokio::time::Instant::now() + debounce;
                while let Ok(Some(Ok(ev))) = tokio::time::timeout_at(deadline, rx.recv()).await {
                    changed.extend(ev.paths.iter().filter_map(|p: &std::path::PathBuf| {
                        p.strip_prefix(&project_root).ok().map(|r: &std::path::Path| r.to_path_buf())
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

                // Notify connected browsers to hot-reload
                let _ = reload_tx.send(());

                // Log successful rebuild if in daemon mode
                if daemon::is_daemon() {
                    let _ = daemon::append_build_log("component", true, None);
                }
            }
            Err(e) => {
                let hint = extract_error_hint(e.to_string());
                *last_build_line = format_last_build_line(false, elapsed, Some(&hint));
                print_status_panel(port, output_dir, Some(last_build_line), None);
                eprintln!("  ✗  {}", e);

                // Log failed build if in daemon mode
                if daemon::is_daemon() {
                    let _ = daemon::append_build_log("component", false, Some(&e.to_string()));
                }
            }
        }
    }

    Ok(())
}

/// Extract a concise one-line description from a build error for the TUI status bar.
#[cfg(feature = "dev-server")]
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
