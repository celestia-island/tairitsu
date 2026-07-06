fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let wit_dir = std::path::Path::new(&manifest_dir).join("wit");
    if wit_dir.exists() {
        println!("cargo:wit_dir={}", wit_dir.display());
        println!(
            "cargo:wit_composed_dir={}",
            wit_dir.join("composed").display()
        );
    }
    println!("cargo:rerun-if-changed=wit/");
}
