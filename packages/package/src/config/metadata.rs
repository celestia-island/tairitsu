use super::{AssetsConfig, BuildConfig, CssConfig, DevConfig, HtmlConfig, NativeConfig};
use serde::Deserialize;

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
                crate::TairitsuPackageError::InvalidConfig(format!(
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
