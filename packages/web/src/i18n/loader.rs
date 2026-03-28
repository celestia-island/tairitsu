//! # I18n Loader
//!
//! Load and parse TOML language files.

use anyhow::{Context, Result};

use crate::keys::I18nKeys;

/// Load TOML file for a specific language
pub fn load_toml(toml_content: &str) -> Result<I18nKeys> {
    toml::from_str(toml_content).context("Failed to parse TOML i18n file")
}

/// Load TOML file for a specific language (static version)
pub fn load_toml_static(toml_content: &'static str) -> Result<I18nKeys> {
    toml::from_str(toml_content).context("Failed to parse TOML i18n file")
}
