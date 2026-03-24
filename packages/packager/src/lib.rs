//! Tairitsu Packager - Build and packaging tool
//!
//! A unified build tool for Tairitsu applications with component-model-first flow.
//! Uses Cargo.toml metadata for configuration instead of HTML templates.

pub mod cli;
pub mod config;
pub mod doctor;
pub mod i18n;
pub mod icons;
pub mod resources;
pub mod ssr;
pub mod styles;
pub mod utils;
pub mod wasm;
pub mod wit_cmd;

pub use cli::run;
pub use config::{Config, TairitsuMetadata};
pub use icons::{
    IconBuildResult, IconConfig, IconMetadata, IconSource, IconStyle, IconsConfig, MdiIconMeta,
    MdiMetadata,
};
pub use resources::{ResourceIndex, ResourceIndexer, ScssResource, ScssUtils, SvgResource, SvgUtils};
pub use utils::error::{Result, TairitsuPackagerError};

/// Version of tairitsu-packager
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
