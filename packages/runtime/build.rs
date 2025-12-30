fn main() {
    // WIT bindings are generated at compile time via wit_bindgen macro
    // This build script is here for future use if needed
    println!("cargo:rerun-if-changed=../../wit");
}
