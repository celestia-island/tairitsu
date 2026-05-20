//! Icon support module for tairitsu-packager.
//!
//! This module provides icon fetching, caching, and code generation capabilities.
//! It allows projects to declaratively configure icons via Cargo.toml metadata.
//!
//! # Example Cargo.toml Configuration
//!
//! ```toml
//! [package.metadata.tairitsu.icons]
//! source = "mdi"
//! icons = ["moon-waning-crescent", "sun", "account"]
//! tags = ["Nature", "Account"]
//! styles = ["filled", "outline"]
//! output = "src/generated/icons.rs"
//! ```
//!
//! # CLI Usage
//!
//! ```bash
//! tairitsu icons fetch    # Download icons from source
//! tairitsu icons build    # Generate Rust code
//! tairitsu icons list     # List available icons
//! ```

mod fetcher;
mod generator;
pub mod hikari_resolver;
mod metadata;

use std::path::PathBuf;

pub use fetcher::{fetch_icons, force_fetch_icons, IconFetcher};
pub use generator::{generate_icon_module, IconBuildResult};
pub use metadata::{
    parse_icons_config, parse_mdi_metadata, IconEntry, IconMetadata, IconsConfig, MdiIconMeta,
    MdiMetadata,
};
use serde::{Deserialize, Serialize};

/// Icon source library
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IconSource {
    /// Material Design Icons (7,447+ icons)
    #[default]
    Mdi,
    /// Lucide icons (~500 icons)
    Lucide,
    /// Custom local SVG files
    Custom,
}

impl std::fmt::Display for IconSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconSource::Mdi => write!(f, "mdi"),
            IconSource::Lucide => write!(f, "lucide"),
            IconSource::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for IconSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mdi" | "materialdesign" | "material-design" => Ok(IconSource::Mdi),
            "lucide" => Ok(IconSource::Lucide),
            "custom" => Ok(IconSource::Custom),
            _ => Err(format!("Unknown icon source: {}", s)),
        }
    }
}

/// Icon style variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IconStyle {
    /// Filled/solid variant
    #[default]
    Filled,
    /// Outlined variant
    Outline,
}

impl std::fmt::Display for IconStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconStyle::Filled => write!(f, "filled"),
            IconStyle::Outline => write!(f, "outline"),
        }
    }
}

/// Icon selection configuration
#[derive(Debug, Clone, Deserialize)]
pub struct IconConfig {
    /// Icon source library
    pub source: IconSource,
    /// Specific icon names to include
    #[serde(default)]
    pub names: Vec<String>,
    /// Tags to include (all icons with these tags)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Style variants to include
    #[serde(default)]
    pub styles: Vec<IconStyle>,
    /// Output path for generated code
    #[serde(default = "default_output")]
    pub output: PathBuf,
}

fn default_output() -> PathBuf {
    PathBuf::from("src/generated/icons.rs")
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            source: IconSource::Mdi,
            names: Vec::new(),
            tags: Vec::new(),
            styles: vec![IconStyle::Filled],
            output: default_output(),
        }
    }
}

/// Cache directory name for icons
pub const ICON_CACHE_DIR: &str = "tairitsu/icons";

/// Default MDI version to use
pub const MDI_DEFAULT_VERSION: &str = "7.4.47";

/// MDI GitHub release URL template
pub const MDI_RELEASE_URL: &str =
    "https://github.com/Templarian/MaterialDesign/releases/download/v{version}/pfd.svg";

/// MDI metadata URL template
pub const MDI_META_URL: &str =
    "https://raw.githubusercontent.com/Templarian/MaterialDesign/master/meta.json";

/// Build icons from configuration
///
/// This is the main entry point for icon building:
/// 1. Downloads icons if not cached
/// 2. Filters by names, tags, styles
/// 3. Generates Rust code
pub fn build_icons(
    config: &IconConfig,
    target_dir: &std::path::Path,
) -> crate::Result<IconBuildResult> {
    let cache_dir = target_dir
        .join(ICON_CACHE_DIR)
        .join(config.source.to_string());

    // Ensure cache directory exists
    std::fs::create_dir_all(&cache_dir)?;

    // Fetch icons (uses cache if available)
    let metadata = fetcher::fetch_icons(&config.source, &cache_dir)?;

    // Filter icons based on configuration
    let selected_icons = metadata.filter_icons(&config.names, &config.tags);

    // Generate Rust code
    let code = generator::generate_icon_module(&selected_icons, &config.styles)?;

    // Write to output file
    if let Some(parent) = config.output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&config.output, &code)?;

    Ok(IconBuildResult {
        icons_count: selected_icons.len(),
        output_path: config.output.clone(),
        generated_code: code,
    })
}
