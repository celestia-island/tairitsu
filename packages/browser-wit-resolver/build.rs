fn main() {
    // Trigger re-run if the WIT cache directory changes.
    // The cache lives at `target/tairitsu-wit` (git-ignored).
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TAIRITSU_WIT_REGISTRY");
    println!("cargo:rerun-if-env-changed=TAIRITSU_WIT_VERSION");
}
