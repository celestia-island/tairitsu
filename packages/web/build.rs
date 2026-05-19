use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let cargo_toml = manifest_dir.join("Cargo.toml");

    let output_dir = resolve_output_dir(&cargo_toml)
        .unwrap_or_else(|| PathBuf::from("../../target/tairitsu-dist"));

    let resolved = if output_dir.is_absolute() {
        output_dir
    } else {
        manifest_dir.join(output_dir)
    };

    println!("cargo:rustc-env=TAIRITSU_DIST_DIR={}", resolved.display());

    resolve_wit_path(&manifest_dir);
}

fn resolve_wit_path(manifest_dir: &Path) {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));

    let Some(wit_composed_dir) = resolve_wit_composed_dir(manifest_dir) else {
        eprintln!(
            "[tairitsu-web] Warning: Could not locate WIT composed directory. \
             wit-bindings feature may not compile."
        );
        return;
    };

    let escaped_path = wit_composed_dir.to_string_lossy().replace('\\', "\\\\");

    let bindings_code = format!(
        r#"wit_bindgen::generate!({{
    path: "{escaped_path}",
    world: "browser-full",
}});"#
    );

    let dest = out_dir.join("wit_bindings_generated.rs");
    if let Err(e) = fs::write(&dest, &bindings_code) {
        eprintln!("[tairitsu-web] Warning: Failed to write generated WIT bindings file: {e}");
    }
}

fn resolve_wit_composed_dir(manifest_dir: &Path) -> Option<PathBuf> {
    let embedded = manifest_dir.join("wit/composed");
    if embedded.join("browser-full.wit").exists() {
        return Some(embedded);
    }

    let monorepo = manifest_dir.join("../browser-worlds/wit/composed");
    if monorepo.join("browser-full.wit").exists() {
        return Some(monorepo);
    }

    if let Ok(dir) = std::env::var("DEP_TAIRITSU_BROWSER_WORLDS_WIT_COMPOSED_DIR") {
        let path = PathBuf::from(&dir);
        if path.join("browser-full.wit").exists() {
            return Some(path);
        }
    }

    None
}

fn resolve_output_dir(cargo_toml: &PathBuf) -> Option<PathBuf> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    let manifest: toml::Value = toml::from_str(&content).ok()?;

    let output_dir = manifest
        .get("package")?
        .get("metadata")?
        .get("tairitsu")?
        .get("build")?
        .get("output_dir")
        .or_else(|| {
            manifest
                .get("package")?
                .get("metadata")?
                .get("tairitsu")?
                .get("build")?
                .get("output-dir")
        })?
        .as_str()?;

    Some(PathBuf::from(output_dir))
}
