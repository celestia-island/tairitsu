# Versioning Strategy

Versioning spans Rust crates, WIT packages, and browser-glue package.

Rules:
1. Rust crate semver changes follow public API compatibility.
2. WIT package versions evolve independently from wasm-bindgen versions.
3. TAIRITSU_WIT_REGISTRY can override package resolution source.
