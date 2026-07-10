//! Vision analysis tools — screenshot + LLM analysis.

use schemars::JsonSchema;

const DEFAULT_PROMPT: &str = "Analyze this screenshot of a web page. Describe:\n\
1. Overall layout and visual structure\n\
2. Any rendering issues (missing content, broken layout, color mismatches, \
overlapping elements, unreadable text)\n\
3. Specific problems with CSS styling\n\
4. Suggestions for improvement\n\
Be concise and specific. Reference CSS classes or element types where relevant.";

/// Capture a screenshot from a running shirabe debug server, or return an error.
async fn capture_screenshot(shirabe_url: &str, full_page: bool) -> anyhow::Result<(String, String)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let resp = client
        .post(format!("{shirabe_url}/screenshot"))
        .header("content-type", "application/json")
        .json(&serde_json::json!({"full_page": full_page}))
        .send()
        .await?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "shirabe screenshot failed (HTTP {}). Is shirabe running at {}?",
            resp.status(),
            shirabe_url
        );
    }

    let json: serde_json::Value = resp.json().await?;
    let data = json["data"]["data"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("unexpected screenshot response format"))?;
    let mime = json["data"]["mime_type"]
        .as_str()
        .unwrap_or("image/png")
        .to_string();

    Ok((data.to_string(), mime))
}

/// Find the provider-registry directory by searching up from CARGO_MANIFEST_DIR.
fn find_registry() -> Option<std::path::PathBuf> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let start = std::path::Path::new(manifest);
    // tairitsu is at celestia/tairitsu/, registry is at celestia/provider-registry/
    let celestia_root = start.parent()?; // celestia/
    let registry = celestia_root.join("provider-registry");
    if registry.is_dir() {
        Some(registry)
    } else {
        None
    }
}

/// Execute the analyze_screenshot flow.
pub async fn do_analyze(
    prompt: Option<String>,
    shirabe_url: Option<String>,
    model: Option<String>,
    full_page: Option<bool>,
) -> anyhow::Result<String> {
    let prompt = prompt.unwrap_or_else(|| DEFAULT_PROMPT.to_string());
    let shirabe_url = shirabe_url
        .unwrap_or_else(|| "http://127.0.0.1:3001".to_string());
    let full_page = full_page.unwrap_or(false);

    // 1. Capture screenshot.
    tracing::info!("capturing screenshot from {shirabe_url}...");
    let (image_b64, mime) = capture_screenshot(&shirabe_url, full_page).await?;
    tracing::info!("screenshot captured ({} bytes base64)", image_b64.len());

    // 2. Load vision models from registry.
    let registry_path = find_registry()
        .ok_or_else(|| anyhow::anyhow!(
            "provider-registry not found. Expected at celestia/provider-registry/"
        ))?;
    let models = crate::registry::load_vision_models(&registry_path);
    if models.is_empty() {
        anyhow::bail!("no vision models found in provider-registry");
    }
    tracing::info!("found {} vision models", models.len());

    // 3. Pick model.
    let chosen = if let Some(model_id) = &model {
        models
            .iter()
            .find(|m| &m.model_id == model_id)
            .ok_or_else(|| anyhow::anyhow!("model '{model_id}' not found in registry"))?
    } else {
        crate::registry::pick_model(&models)
            .ok_or_else(|| anyhow::anyhow!("no suitable model found"))?
    };
    tracing::info!(
        "using model: {} (provider: {}, ${:.2}/M input)",
        chosen.model_id,
        chosen.provider_id,
        chosen.input_price
    );

    // 4. Call vision API.
    tracing::info!("sending analysis request...");
    let result = crate::client::analyze_image(chosen, &prompt, &image_b64, &mime).await?;
    tracing::info!("analysis complete ({} chars)", result.len());

    Ok(result)
}

/// Execute list_vision_models.
pub fn do_list_models() -> anyhow::Result<Vec<ModelInfo>> {
    let registry_path = find_registry()
        .ok_or_else(|| anyhow::anyhow!(
            "provider-registry not found. Expected at celestia/provider-registry/"
        ))?;
    let models = crate::registry::load_vision_models(&registry_path);
    Ok(models
        .iter()
        .map(|m| ModelInfo {
            model_id: m.model_id.clone(),
            provider: m.provider_id.clone(),
            input_price: m.input_price,
            env_var: m.env_var.clone(),
            has_key: m.api_key().is_some(),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ModelInfo {
    pub model_id: String,
    pub provider: String,
    pub input_price: f64,
    pub env_var: String,
    pub has_key: bool,
}

use serde::Serialize;
