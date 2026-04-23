mod metadata;
mod routes;

pub use routes::{discover_routes, DiscoveredRoute};

use serde::Deserialize;
use std::path::{Path, PathBuf};

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
    pub scss: ScssConfig,
    #[serde(default)]
    pub native: NativeConfig,
    /// Directory containing the Cargo.toml (project root)
    #[serde(skip)]
    pub manifest_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PackageConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
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
    #[serde(alias = "browser-glue-path")]
    #[serde(default = "default_browser_glue_path")]
    pub browser_glue_path: String,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: default_target(),
            output_dir: default_output_dir(),
            optimize: false,
            sourcemap: false,
            browser_glue_path: default_browser_glue_path(),
        }
    }
}

fn default_target() -> String {
    "component".to_string()
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("../../target/tairitsu-dist")
}

fn default_browser_glue_path() -> String {
    "/browser-glue/__tairitsu_glue__.js".to_string()
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
    3000
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

#[derive(Debug, Clone, Deserialize)]
pub struct AssetsConfig {
    #[serde(default = "default_inline_limit")]
    pub inline_limit: usize,
    #[serde(default)]
    pub include: Vec<String>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    #[serde(alias = "extra-public-dirs")]
    pub extra_public_dirs: Vec<String>,
}

impl Default for AssetsConfig {
    fn default() -> Self {
        Self {
            inline_limit: default_inline_limit(),
            include: Vec::new(),
            exclude: Vec::new(),
            extra_public_dirs: Vec::new(),
        }
    }
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

/// SCSS compilation configuration
#[derive(Debug, Clone, Deserialize)]
pub struct ScssConfig {
    /// Single entry point (legacy, for backward compatibility)
    #[serde(default)]
    pub entry: Option<String>,
    /// Single output filename (legacy)
    #[serde(default = "default_scss_output")]
    pub output: String,
    /// Multiple entry points
    #[serde(default)]
    pub entries: Vec<ScssEntry>,
    /// Load paths for @use and @import
    #[serde(default)]
    pub load_paths: Vec<String>,
}

impl Default for ScssConfig {
    fn default() -> Self {
        Self {
            entry: None,
            output: default_scss_output(),
            entries: Vec::new(),
            load_paths: Vec::new(),
        }
    }
}

fn default_scss_output() -> String {
    "styles.css".to_string()
}

/// Single SCSS entry point configuration
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ScssEntry {
    /// Entry SCSS file path
    pub entry: String,
    /// Output CSS filename
    pub output: String,
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

        // Get the directory containing Cargo.toml, canonicalized to absolute path
        let manifest_dir = cargo_toml_path
            .parent()
            .map(|p| p.canonicalize().unwrap_or_else(|_| p.to_path_buf()))
            .unwrap_or_default();

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
            scss: metadata.scss,
            native: metadata.native,
            manifest_dir,
        })
    }

    /// Auto-discover client-side routes from the `html.head` JavaScript.
    ///
    /// Parses `const ROUTES = { ... }` or similar patterns embedded in the
    /// head field and returns structured route mappings. This enables:
    /// - Automatic SSG (per-route HTML generation during build)
    /// - Route-aware SSR rendering
    /// - Dev server route discovery without manual --routes flags
    pub fn discovered_routes(&self) -> Vec<DiscoveredRoute> {
        discover_routes(&self.html.head)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_build_config() {
        let config = BuildConfig::default();
        assert_eq!(config.target, "component");
        assert_eq!(
            config.output_dir.to_str().unwrap(),
            "../../target/tairitsu-dist"
        );
        assert!(!config.optimize);
        assert!(!config.sourcemap);
        assert_eq!(
            config.browser_glue_path,
            "/browser-glue/__tairitsu_glue__.js"
        );
    }

    #[test]
    fn test_default_dev_config() {
        let config = DevConfig::default();
        assert_eq!(config.port, 3001);
        assert!(config.hot_reload);
        assert!(!config.open_browser);
    }

    #[test]
    fn test_default_html_config() {
        let config = HtmlConfig::default();
        assert_eq!(config.lang, "en");
        assert_eq!(config.charset, "UTF-8");
        assert_eq!(config.viewport, "width=device-width, initial-scale=1.0");
        assert!(config.favicon.is_none());
        assert!(config.title.is_none());
        assert!(config.head.is_empty());
        assert!(config.body_class.is_empty());
    }

    #[test]
    fn test_default_assets_config() {
        let config = AssetsConfig::default();
        assert_eq!(config.inline_limit, 8192);
        assert!(config.include.is_empty());
        assert!(config.exclude.is_empty());
    }

    #[test]
    fn test_default_css_config() {
        let config = CssConfig::default();
        assert!(config.files.is_empty());
        assert!(!config.autoprefixer);
        assert!(!config.minify);
    }

    #[test]
    fn test_default_scss_config() {
        let config = ScssConfig::default();
        assert!(config.entry.is_none());
        assert_eq!(config.output, "styles.css");
        assert!(config.entries.is_empty());
        assert!(config.load_paths.is_empty());
    }

    #[test]
    fn test_default_native_config() {
        let config = NativeConfig::default();
        assert!(config.identifier.is_none());
        assert!(config.icon.is_none());
        assert!(config.copyright.is_none());
    }

    #[test]
    fn test_scss_entry_default() {
        let entry = ScssEntry::default();
        assert!(entry.entry.is_empty());
        assert!(entry.output.is_empty());
    }

    #[test]
    fn test_build_config_aliases() {
        let toml_str = r#"
            target = "app"
            output-dir = "custom/output"
            browser-glue-path = "/custom/glue.js"
            optimize = true
            sourcemap = true
        "#;

        let config: BuildConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.target, "app");
        assert_eq!(config.output_dir, PathBuf::from("custom/output"));
        assert_eq!(config.browser_glue_path, "/custom/glue.js");
        assert!(config.optimize);
        assert!(config.sourcemap);
    }

    #[test]
    fn test_load_minimal_config() {
        let toml_content = r#"
[package]
name = "test-app"
version = "1.0.0"
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, toml_content).unwrap();

        let config = Config::load(temp_dir.path()).unwrap();
        assert_eq!(config.package.name, "test-app");
        assert_eq!(config.package.version, "1.0.0");
        assert_eq!(config.build.target, "component"); // default from BuildConfig
        assert_eq!(config.dev.port, 3001); // default
    }

    #[test]
    fn test_load_full_config() {
        let toml_content = r#"
[package]
name = "full-app"
version = "2.0.0"

[package.metadata.tairitsu.build]
target = "app"
optimize = true
sourcemap = true

[package.metadata.tairitsu.dev]
port = 4000
hot_reload = false
open_browser = true

[package.metadata.tairitsu.html]
lang = "zh"
title = "Test App"
favicon = "/favicon.ico"

[package.metadata.tairitsu.css]
files = ["styles/main.css"]
minify = true

[package.metadata.tairitsu.scss]
entry = "styles/main.scss"
output = "compiled.css"
load_paths = ["node_modules"]

[package.metadata.tairitsu.native]
identifier = "com.example.test"
icon = "icon.ico"
copyright = "2024 Test"
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, toml_content).unwrap();

        let config = Config::load(temp_dir.path()).unwrap();
        assert_eq!(config.package.name, "full-app");
        assert_eq!(config.package.version, "2.0.0");
        assert_eq!(config.build.target, "app");
        assert!(config.build.optimize);
        assert!(config.build.sourcemap);
        assert_eq!(config.dev.port, 4000);
        assert!(!config.dev.hot_reload);
        assert!(config.dev.open_browser);
        assert_eq!(config.html.lang, "zh");
        assert_eq!(config.html.title, Some("Test App".to_string()));
        assert_eq!(config.html.favicon, Some("/favicon.ico".to_string()));
        assert_eq!(config.css.files, vec!["styles/main.css".to_string()]);
        assert!(config.css.minify);
        assert_eq!(config.scss.entry, Some("styles/main.scss".to_string()));
        assert_eq!(config.scss.output, "compiled.css");
        assert_eq!(config.scss.load_paths, vec!["node_modules".to_string()]);
        assert_eq!(
            config.native.identifier,
            Some("com.example.test".to_string())
        );
        assert_eq!(config.native.icon, Some("icon.ico".to_string()));
        assert_eq!(config.native.copyright, Some("2024 Test".to_string()));
    }

    #[test]
    fn test_load_config_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = Config::load(temp_dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::TairitsuPackagerError::ConfigNotFound(_) => {}
            _ => panic!("Expected ConfigNotFound error"),
        }
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let toml_content = r#"
[package
name = "invalid"
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, toml_content).unwrap();

        let result = Config::load(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_missing_package_section() {
        let toml_content = r#"
[workspace]
members = ["."]
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, toml_content).unwrap();

        let result = Config::load(temp_dir.path());
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::TairitsuPackagerError::InvalidConfig(msg) => {
                assert!(msg.contains("Missing [package] section"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_scss_entries_config() {
        let toml_str = r#"
[[entries]]
entry = "styles/main.scss"
output = "main.css"

[[entries]]
entry = "styles/admin.scss"
output = "admin.css"
"#;

        let config: ScssConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.entries.len(), 2);
        assert_eq!(config.entries[0].entry, "styles/main.scss");
        assert_eq!(config.entries[0].output, "main.css");
        assert_eq!(config.entries[1].entry, "styles/admin.scss");
        assert_eq!(config.entries[1].output, "admin.css");
    }

    #[test]
    fn test_assets_config_with_patterns() {
        let toml_str = r#"
inline_limit = 4096
include = ["**/*.png", "**/*.jpg"]
exclude = ["**/*.min.css", "**/node_modules/**"]
"#;

        let config: AssetsConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.inline_limit, 4096);
        assert_eq!(config.include.len(), 2);
        assert_eq!(config.exclude.len(), 2);
    }

    #[test]
    fn test_load_with_file_path() {
        let toml_content = r#"
[package]
name = "path-test"
version = "1.0.0"
"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");
        std::fs::write(&cargo_toml_path, toml_content).unwrap();

        // Test loading with file path instead of directory
        let config = Config::load(&cargo_toml_path).unwrap();
        assert_eq!(config.package.name, "path-test");
    }
}
