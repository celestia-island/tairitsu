use std::fs;
use std::path::PathBuf;

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
