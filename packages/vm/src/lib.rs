//! Tairitsu - A WIT-based bidirectional communication framework for WASM
//!
//! This framework provides Docker-like image/container mechanisms with
//! bidirectional communication between host and guest WASM modules.

mod commands;
mod container;
mod image;
mod registry;

pub use commands::{
    deserialize_command, serialize_command, GuestCommands, GuestResponse, HostCommands,
    HostResponse, LogLevel,
};
pub use container::Container;
pub use image::Image;
pub use registry::Registry;

// Re-export common types
pub use anyhow::{Error, Result};

wit_bindgen::generate!({
    path: "../../wit",
    world: "tairitsu",
    generate_all
});
