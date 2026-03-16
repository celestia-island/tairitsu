fn main() {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"),
    );
    let template_js = manifest_dir
        .join("src")
        .join("wasm")
        .join("component-wrapper-loader.template.ts");
    println!("cargo:rerun-if-changed={}", template_js.display());
}
