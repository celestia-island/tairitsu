//! Build script template for automatic resource indexing.
//!
//! Copy this file to your project's `build.rs` to enable automatic resource
//! indexing during builds. This will scan your project for SCSS and SVG files
//! and generate a resource index with content hashes for cache busting.
//!
//! # Usage
//!
//! 1. Copy this file to your project root as `build.rs`
//! 2. Add `tairitsu-packager` to your build dependencies in `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! tairitsu-packager = "0.1"
//! ```
//!
//! 3. The index will be generated at `target/tairitsu/resources/index.json`
//!    during each build.

use std::path::PathBuf;

fn main() {
    // Get the project manifest directory
    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR missing")
    );

    // Get the target directory (uses CARGO_TARGET_DIR or falls back to manifest_dir/target)
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));

    // Create the resource indexer
    let indexer = tairitsu_packager::resources::ResourceIndexer::new(&manifest_dir);

    // Index resources and save to target directory
    match indexer.index_to_target(&target_dir) {
        Ok((index, output_path)) => {
            println!("cargo:warning=Indexed {} resources to {}",
                index.count(),
                output_path.display()
            );

            // Tell Cargo to rerun this script if any resource files change
            for scss in &index.scss {
                println!("cargo:rerun-if-changed={}", scss.source);
            }
            for svg in &index.svg {
                println!("cargo:rerun-if-changed={}", svg.source);
            }
        }
        Err(e) => {
            // Don't fail the build, but warn about the error
            println!("cargo:warning=Failed to index resources: {}", e);
        }
    }

    // Also watch the Cargo.toml for configuration changes
    println!("cargo:rerun-if-changed=Cargo.toml");
}
