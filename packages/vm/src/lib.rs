//! Tairitsu - A WIT-based bidirectional communication framework for WASM
//! 
//! This framework provides Docker-like image/container mechanisms with
//! bidirectional communication between host and guest WASM modules.

mod image;
mod container;
mod registry;

pub use image::Image;
pub use container::Container;
pub use registry::Registry;

// Re-export common types
pub use anyhow::{Error, Result};

wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
    generate_all
});
