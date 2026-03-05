//! Tairitsu Packager - Build and packaging tool
//!
//! A unified build tool for Tairitsu applications, replacing trunk and tauri-build.
//! Uses Cargo.toml metadata for configuration instead of HTML templates.

pub mod cli;
pub mod config;
pub mod wasm;
pub mod utils;

pub use cli::run;
pub use config::{Config, TairitsuMetadata};
pub use utils::error::{Result, TairitsuPackagerError};

/// Version of tairitsu-packager
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
