use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = find_workspace_root(&manifest_dir);

    // Watch for template changes
    let template_js = manifest_dir.join("src/wasm/component-wrapper-loader.template.js");
    println!("cargo:rerun-if-changed={}", template_js.display());

    // Watch for runtime changes (all files in the runtime/ folder)
    let runtime_dir = workspace_root.join("packages/browser-glue/src/runtime");
    if runtime_dir.is_dir() {
        for entry in std::fs::read_dir(&runtime_dir)
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("ts") {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    } else {
        println!(
            "cargo:rerun-if-changed={}",
            runtime_dir.with_extension("ts").display()
        );
    }

    // Watch for glue files in src/glue/
    let glue_files = [
        "console.ts",
        "style.ts",
        "event-target.ts",
        "css.ts",
        "dom.ts",
    ];
    for file in &glue_files {
        let path = workspace_root
            .join("packages/browser-glue/src/glue")
            .join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    // Also watch the shared helpers
    for file in &["handles.ts", "async.ts"] {
        let path = workspace_root.join("packages/browser-glue/src").join(file);
        println!("cargo:rerun-if-changed={}", path.display());
    }

    // Compile runtime via esbuild → dist/runtime.js (read at runtime, not embedded)
    compile_runtime(&workspace_root);
}

fn compile_runtime(workspace_root: &Path) {
    let src_file = workspace_root.join("packages/browser-glue/src/runtime/index.ts");
    let out_file = workspace_root.join("packages/browser-glue/dist/runtime.js");

    if let Some(parent) = out_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let npx = find_npx();

    let output = Command::new(&npx)
        .args([
            "esbuild",
            &src_file.to_string_lossy(),
            "--bundle",
            &format!("--outfile={}", out_file.to_string_lossy()),
            "--format=iife",
            "--platform=browser",
        ])
        .current_dir(workspace_root.join("packages/browser-glue"))
        .output();

    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            println!("cargo:warning=esbuild failed (status={}):", output.status);
            println!(
                "cargo:warning=  stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(e) => {
            println!("cargo:warning=esbuild command '{}' not found: {}", npx, e);
        }
    }
}

fn find_npx() -> String {
    #[cfg(target_os = "windows")]
    {
        if Command::new("npx.cmd").arg("--version").output().is_ok() {
            return "npx.cmd".to_string();
        }
    }
    if let Ok(output) = Command::new("npm").args(["root", "-g"]).output() {
        let npm_root = String::from_utf8_lossy(&output.stdout).trim().to_string();
        #[cfg(target_os = "windows")]
        let npx_path = format!("{}\\npx.cmd", npm_root);
        #[cfg(not(target_os = "windows"))]
        let npx_path = format!("{}/npx", npm_root);
        if std::path::Path::new(&npx_path).exists() {
            return npx_path;
        }
    }
    "npx".to_string()
}

fn find_workspace_root(manifest_dir: &Path) -> PathBuf {
    let mut current = manifest_dir.parent();
    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            if let Ok(cargo_toml) = std::fs::read_to_string(dir.join("Cargo.toml")) {
                if cargo_toml.contains("[workspace]") {
                    return dir.to_path_buf();
                }
            }
        }
        current = dir.parent();
    }
    manifest_dir.parent().unwrap().to_path_buf()
}
