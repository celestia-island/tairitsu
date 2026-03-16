use serde::Deserialize;
use std::path::{Path, PathBuf};

mod metadata;

pub use metadata::TairitsuMetadata;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub package: PackageConfig,
    pub build: BuildConfig,
    pub dev: DevConfig,
    pub assets: AssetsConfig,
    pub html: HtmlConfig,
    #[serde(default)]
    pub css: CssConfig,
    #[serde(default)]
    pub native: NativeConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct BuildConfig {
    #[serde(default = "default_target")]
    pub target: String,
    #[serde(alias = "output-dir")]
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default)]
    pub optimize: bool,
    #[serde(default)]
    pub sourcemap: bool,
}

fn default_target() -> String {
    "component".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("../../target/tairitsu-dist")
}

#[derive(Debug, Clone, Deserialize)]
pub struct DevConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_hot_reload")]
    pub hot_reload: bool,
    #[serde(default)]
    pub open_browser: bool,
}

fn default_port() -> u16 {
    3001
}

fn default_hot_reload() -> bool {
    true
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            hot_reload: default_hot_reload(),
            open_browser: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssetsConfig {
    #[serde(default = "default_inline_limit")]
    pub inline_limit: usize,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
}

fn default_inline_limit() -> usize {
    8192
}

#[derive(Debug, Clone, Deserialize)]
pub struct HtmlConfig {
    #[serde(default = "default_lang")]
    pub lang: String,
    #[serde(default = "default_charset")]
    pub charset: String,
    #[serde(default = "default_viewport")]
    pub viewport: String,
    #[serde(default)]
    pub favicon: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub head: String,
    #[serde(default)]
    pub body_class: String,
}

fn default_lang() -> String {
    "en".to_string()
}

fn default_charset() -> String {
    "UTF-8".to_string()
}

fn default_viewport() -> String {
    "width=device-width, initial-scale=1.0".to_string()
}

impl Default for HtmlConfig {
    fn default() -> Self {
        Self {
            lang: default_lang(),
            charset: default_charset(),
            viewport: default_viewport(),
            favicon: None,
            title: None,
            head: String::new(),
            body_class: String::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CssConfig {
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub autoprefixer: bool,
    #[serde(default)]
    pub minify: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct NativeConfig {
    #[serde(default)]
    pub identifier: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub copyright: Option<String>,
}

impl Config {
    pub fn load(manifest_path: &Path) -> crate::Result<Self> {
        let cargo_toml_path = if manifest_path.is_dir() {
            manifest_path.join("Cargo.toml")
        } else {
            manifest_path.to_path_buf()
        };

        if !cargo_toml_path.exists() {
            return Err(crate::TairitsuPackagerError::ConfigNotFound(
                cargo_toml_path.display().to_string(),
            ));
        }

        let content = std::fs::read_to_string(&cargo_toml_path)?;
        let manifest: toml::Value = toml::from_str(&content)?;

        // Extract package info
        let package = manifest.get("package").ok_or_else(|| {
            crate::TairitsuPackagerError::InvalidConfig("Missing [package] section".to_string())
        })?;

        let name = package
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("app")
            .to_string();

        let version = package
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .to_string();

        // Extract tairitsu metadata
        let metadata = metadata::parse_metadata(&manifest)?;

        Ok(Config {
            package: PackageConfig { name, version },
            build: metadata.build,
            dev: metadata.dev,
            assets: metadata.assets,
            html: metadata.html,
            css: metadata.css,
            native: metadata.native,
        })
    }
}
