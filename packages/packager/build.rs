fn main() {
    let template_ts = std::path::Path::new("src")
        .join("wasm")
        .join("component-wrapper-loader.template.ts");
    let out_js = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"))
        .join("component-wrapper-loader.template.js");

    println!("cargo:rerun-if-changed={}", template_ts.display());

    let source = std::fs::read_to_string(&template_ts)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", template_ts.display()));

    std::fs::write(&out_js, source)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", out_js.display()));
}