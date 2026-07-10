//! Vision model registry — loads `supports_vision = true` models from the
//! provider-registry TOML files at runtime.
//!
//! The registry is located at `<celestia>/provider-registry/`. Each model is
//! a TOML file under `models/<provider>/<model>.toml`; each provider's API
//! config is under `entrypoint/<provider>/default.toml`.

use serde::Deserialize;

/// A vision-capable model entry loaded from provider-registry.
#[derive(Debug, Clone, Deserialize)]
pub struct VisionModel {
    #[serde(rename = "model")]
    pub inner: ModelInner,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInner {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    #[serde(default)]
    pub supports_vision: bool,
    #[serde(default)]
    pub context_window: u64,
    #[serde(default)]
    pub pricing: ModelPricing,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ModelPricing {
    #[serde(default)]
    pub input_per_million: f64,
    #[serde(default)]
    pub output_per_million: f64,
}

/// A provider's API entrypoint config.
#[derive(Debug, Clone, Deserialize)]
pub struct ProviderEntry {
    #[serde(rename = "entrypoint")]
    pub inner: EntrypointInner,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EntrypointInner {
    pub provider_id: String,
    #[serde(rename = "api")]
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub protocol: String,
    pub base_url: String,
    #[serde(default)]
    pub chat_endpoint: String,
    #[serde(default = "default_bearer")]
    pub auth_type: String,
    #[serde(default = "default_auth_header")]
    pub auth_header: String,
    #[serde(default)]
    pub env_var: String,
}

fn default_bearer() -> String {
    "bearer".to_string()
}

fn default_auth_header() -> String {
    "Authorization".to_string()
}

/// Combined model + API info for making a vision request.
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    pub model_id: String,
    pub provider_id: String,
    pub base_url: String,
    pub chat_endpoint: String,
    pub protocol: String,
    pub auth_type: String,
    pub auth_header: String,
    pub env_var: String,
    pub input_price: f64,
}

impl ResolvedModel {
    /// Resolve the API key from the environment.
    pub fn api_key(&self) -> Option<String> {
        // Try the provider-specific env var first, then generic fallbacks.
        if !self.env_var.is_empty() {
            if let Ok(key) = std::env::var(&self.env_var) {
                if !key.is_empty() {
                    return Some(key);
                }
            }
        }
        // Generic fallbacks.
        for var in &["LAGRANGE_VISION_API_KEY", "OPENAI_API_KEY", "OPENROUTER_API_KEY"] {
            if let Ok(key) = std::env::var(var) {
                if !key.is_empty() {
                    return Some(key);
                }
            }
        }
        None
    }

    /// Build the full chat completions URL.
    pub fn chat_url(&self) -> String {
        let base = self.base_url.trim_end_matches('/');
        let endpoint = if self.chat_endpoint.is_empty() {
            "/chat/completions"
        } else {
            &self.chat_endpoint
        };
        format!("{base}{endpoint}")
    }

    /// Build the auth header value.
    pub fn auth_value(&self, key: &str) -> String {
        if self.auth_type == "bearer" {
            format!("Bearer {key}")
        } else {
            key.to_string()
        }
    }
}

/// Scan provider-registry for vision models and resolve API configs.
///
/// `registry_path` should point to the `provider-registry/` directory.
/// Returns a list of resolved models sorted by price (cheapest first).
pub fn load_vision_models(registry_path: &std::path::Path) -> Vec<ResolvedModel> {
    let models_dir = registry_path.join("models");
    let entrypoint_dir = registry_path.join("entrypoint");

    // Load all entrypoints into a map: provider_id → ApiConfig.
    let mut entrypoints: std::collections::HashMap<String, ApiConfig> = std::collections::HashMap::new();
    if entrypoint_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&entrypoint_dir) {
            for entry in entries.flatten() {
                let toml_path = entry.path().join("default.toml");
                if toml_path.is_file() {
                    if let Ok(content) = std::fs::read_to_string(&toml_path) {
                        if let Ok(ProviderEntry { inner }) = toml::from_str::<ProviderEntry>(&content)
                        {
                            entrypoints.insert(inner.provider_id.clone(), inner.api);
                        }
                    }
                }
            }
        }
    }

    // Scan all model TOMLs for supports_vision = true.
    let mut models: Vec<ResolvedModel> = Vec::new();
    if models_dir.is_dir() {
        scan_models_dir(&models_dir, &entrypoints, &mut models);
    }

    // Sort by input price (cheapest first).
    models.sort_by(|a, b| {
        a.input_price
            .partial_cmp(&b.input_price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    models
}

fn scan_models_dir(
    dir: &std::path::Path,
    entrypoints: &std::collections::HashMap<String, ApiConfig>,
    out: &mut Vec<ResolvedModel>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_models_dir(&path, entrypoints, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("toml") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(VisionModel { inner: m }) = toml::from_str::<VisionModel>(&content) {
                    if !m.supports_vision {
                        continue;
                    }
                    // Find matching entrypoint.
                    if let Some(api) = entrypoints.get(&m.provider_id) {
                        out.push(ResolvedModel {
                            model_id: m.id,
                            provider_id: m.provider_id.clone(),
                            base_url: api.base_url.clone(),
                            chat_endpoint: api.chat_endpoint.clone(),
                            protocol: api.protocol.clone(),
                            auth_type: api.auth_type.clone(),
                            auth_header: api.auth_header.clone(),
                            env_var: api.env_var.clone(),
                            input_price: m.pricing.input_per_million,
                        });
                    }
                }
            }
        }
    }
}

/// Pick the best model: explicit override → env var → cheapest available.
pub fn pick_model(models: &[ResolvedModel]) -> Option<&ResolvedModel> {
    // 1. User-specified via env var.
    if let Ok(model_id) = std::env::var("LAGRANGE_VISION_MODEL") {
        if let Some(m) = models.iter().find(|m| m.model_id == model_id) {
            return Some(m);
        }
    }
    // 2. Cheapest with an available API key.
    models.iter().find(|m| m.api_key().is_some()).or_else(|| models.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_model() {
        let toml = r#"
[model]
id = "glm-5v-turbo"
name = "GLM-5V-Turbo"
provider_id = "zhipu_glm"
supports_vision = true

[model.pricing]
input_per_million = 1.2
output_per_million = 4.0
"#;
        let vm: VisionModel = toml::from_str(toml).unwrap();
        assert_eq!(vm.inner.id, "glm-5v-turbo");
        assert!(vm.inner.supports_vision);
        assert_eq!(vm.inner.pricing.input_per_million, 1.2);
    }
}
