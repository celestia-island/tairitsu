use serde::Deserialize;

use super::{
    AssetsConfig, BuildConfig, CssConfig, DevConfig, HtmlConfig, NativeConfig, ScssConfig,
};

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TairitsuMetadata {
    #[serde(default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub build: BuildConfig,
    #[serde(default)]
    pub dev: DevConfig,
    #[serde(default)]
    pub assets: AssetsConfig,
    #[serde(default)]
    pub html: HtmlConfig,
    #[serde(default)]
    pub css: CssConfig,
    #[serde(default)]
    pub scss: ScssConfig,
    #[serde(default)]
    pub native: NativeConfig,
}

pub fn parse_metadata(manifest: &toml::Value) -> crate::Result<TairitsuMetadata> {
    let metadata = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("tairitsu"));

    match metadata {
        Some(value) => {
            let metadata: TairitsuMetadata = value.clone().try_into().map_err(|e| {
                crate::TairitsuPackagerError::InvalidConfig(format!(
                    "Invalid tairitsu metadata: {}",
                    e
                ))
            })?;
            Ok(metadata)
        }
        None => {
            // Return defaults if no metadata
            Ok(TairitsuMetadata::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_metadata_none() {
        let toml_str = r#"
[package]
name = "test"
version = "1.0.0"
"#;
        let manifest: toml::Value = toml::from_str(toml_str).unwrap();
        let metadata = parse_metadata(&manifest).unwrap();

        assert!(metadata.app_name.is_none());
        assert!(metadata.title.is_none());
        assert!(metadata.description.is_none());
        // Default values from BuildConfig::default()
        assert_eq!(metadata.build.target, "component");
        assert_eq!(metadata.dev.port, 3001);
    }

    #[test]
    fn test_parse_metadata_with_values() {
        let toml_str = r#"
[package]
name = "test"

[package.metadata.tairitsu]
app_name = "My App"
title = "My Title"
description = "My Description"

[package.metadata.tairitsu.build]
target = "app"
optimize = true

[package.metadata.tairitsu.dev]
port = 5000
"#;
        let manifest: toml::Value = toml::from_str(toml_str).unwrap();
        let metadata = parse_metadata(&manifest).unwrap();

        assert_eq!(metadata.app_name, Some("My App".to_string()));
        assert_eq!(metadata.title, Some("My Title".to_string()));
        assert_eq!(metadata.description, Some("My Description".to_string()));
        assert_eq!(metadata.build.target, "app");
        assert!(metadata.build.optimize);
        assert_eq!(metadata.dev.port, 5000);
    }

    #[test]
    fn test_parse_metadata_invalid_type() {
        let toml_str = r#"
[package]
name = "test"

[package.metadata.tairitsu.build]
target = 123
"#;
        let manifest: toml::Value = toml::from_str(toml_str).unwrap();
        let result = parse_metadata(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_tairitsu_metadata_default() {
        let metadata = TairitsuMetadata::default();
        assert!(metadata.app_name.is_none());
        assert!(metadata.title.is_none());
        assert!(metadata.description.is_none());
        // Default values from Default impl
        assert_eq!(metadata.build.target, "component");
        assert_eq!(metadata.dev.port, 3001);
    }
}
