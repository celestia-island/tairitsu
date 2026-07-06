//! Tairitsu Packager - Build and packaging tool
//!
//! A unified build tool for Tairitsu applications with component-model-first flow.
//! Uses Cargo.toml metadata for configuration instead of HTML templates.

#[cfg(feature = "cli")]
pub mod cli;
pub mod config;
#[cfg(feature = "tokio")]
pub mod daemon;
#[cfg(feature = "dev-server")]
pub mod debug;
pub mod i18n;
pub mod icons;
pub mod logfmt;
#[cfg(feature = "cli")]
pub mod mcp;
pub mod resources;
#[cfg(feature = "ssr")]
pub mod ssr;
pub mod styles;
#[cfg(feature = "test-runner")]
pub mod test_runner;
pub mod utils;
#[cfg(feature = "visual-diff")]
pub mod visual_diff;
#[cfg(feature = "vtty")]
pub mod vtty;
#[cfg(feature = "tokio")]
pub mod wasm;
pub mod wit_check;
pub mod wit_cmd;
pub mod wit_plugin;

#[cfg(feature = "cli")]
pub use cli::run;
pub use config::{Config, TairitsuMetadata};
pub use icons::{
    generate_woff_subset, is_hb_subset_available, CacheManifest, HikariIconsMetadata, IconCache,
    ResolveResult, ResolvedSet, SetConfig, Subscript,
};
pub use resources::{
    ResourceIndex, ResourceIndexer, ScssResource, ScssUtils, SvgResource, SvgUtils,
};
pub use utils::error::{Result, TairitsuPackagerError};
pub use wit_plugin::PluginWitRegistry;

/// Version of tairitsu-packager
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
