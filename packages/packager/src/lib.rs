//! Tairitsu Packager - Build and packaging tool
//!
//! A unified build tool for Tairitsu applications with component-model-first flow.
//! Uses Cargo.toml metadata for configuration instead of HTML templates.

#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
#[cfg(feature = "dev-server")]
pub mod debug;
#[cfg(feature = "visual-diff")]
pub mod visual_diff;
#[cfg(feature = "test-runner")]
pub mod test_runner;
#[cfg(any(feature = "debug-api", feature = "dev-server"))]
pub mod debug_api;
#[cfg(feature = "tokio")]
pub mod daemon;
#[cfg(feature = "cli")]
pub mod mcp;
pub mod i18n;
pub mod icons;
pub mod logfmt;
pub mod resources;
#[cfg(feature = "ssr")]
pub mod ssr;
pub mod styles;
pub mod utils;
#[cfg(feature = "tokio")]
pub mod wasm;
pub mod wit_cmd;
pub mod wit_check;

#[cfg(feature = "cli")]
pub use cli::run;
pub use config::{Config, TairitsuMetadata};
pub use icons::{
    IconBuildResult, IconConfig, IconMetadata, IconSource, IconStyle, IconsConfig, MdiIconMeta,
    MdiMetadata,
};
pub use resources::{
    ResourceIndex, ResourceIndexer, ScssResource, ScssUtils, SvgResource, SvgUtils,
};
pub use utils::error::{Result, TairitsuPackagerError};

/// Version of tairitsu-packager
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
