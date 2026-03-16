fn main() {
    let manifest_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing"),
    );
    let template_ts = manifest_dir
        .join("src")
        .join("wasm")
        .join("component-wrapper-loader.template.ts");
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR missing"));
    let tsc_js = manifest_dir
        .join("..")
        .join("browser-glue")
        .join("node_modules")
        .join("typescript")
        .join("lib")
        .join("tsc.js");

    println!("cargo:rerun-if-changed={}", template_ts.display());

    if !tsc_js.exists() {
        panic!(
            "TypeScript compiler not found at {}. Run `npm install` in packages/browser-glue first.",
            tsc_js.display()
        );
    }

    let output = std::process::Command::new("node")
        .arg(&tsc_js)
        .args([
            "--pretty",
            "false",
            "--target",
            "ES2022",
            "--module",
            "ES2022",
            "--moduleResolution",
            "bundler",
            "--lib",
            "DOM,ES2022",
            "--strict",
            "--skipLibCheck",
            "--outDir",
        ])
        .arg(&out_dir)
        .arg(&template_ts)
        .output()
        .unwrap_or_else(|err| panic!("failed to run node for TypeScript compilation: {err}"));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "TypeScript compilation failed for {}\nstdout:\n{}\nstderr:\n{}",
            template_ts.display(),
            stdout,
            stderr
        );
    }
}