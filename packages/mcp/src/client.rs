//! Lightweight OpenAI-compatible multimodal client.
//!
//! Sends a base64-encoded image + text prompt to a vision model's chat
//! completions endpoint. Only implements the request shape — no streaming,
//! no retries, no tool calling. Designed for one-shot vision analysis.

use serde::Deserialize;

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

/// Send a vision analysis request and return the model's text response.
pub async fn analyze_image(
    model: &crate::registry::ResolvedModel,
    prompt: &str,
    image_base64: &str,
    image_mime: &str,
) -> anyhow::Result<String> {
    let api_key = model.api_key().ok_or_else(|| {
        anyhow::anyhow!(
            "no API key found. Set {} or LAGRANGE_VISION_API_KEY",
            model.env_var
        )
    })?;

    let data_url = format!("data:{image_mime};base64,{image_base64}");
    let body = serde_json::json!({
        "model": model.model_id,
        "messages": [{
            "role": "user",
            "content": [
                {"type": "text", "text": prompt},
                {"type": "image_url", "image_url": {"url": data_url}}
            ]
        }],
        "max_tokens": 4096,
        "temperature": 0.3
    });

    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;

    let resp = client
        .post(model.chat_url())
        .header(&model.auth_header, model.auth_value(&api_key))
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;

    if !status.is_success() {
        let snippet = if text.len() > 500 {
            &text[..500]
        } else {
            &text
        };
        anyhow::bail!("API error {status}: {snippet}");
    }

    let chat_resp: ChatResponse = serde_json::from_str(&text).map_err(|e| {
        anyhow::anyhow!(
            "failed to parse response: {e}\nraw: {}",
            &text[..text.len().min(500)]
        )
    })?;

    chat_resp
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| anyhow::anyhow!("no choices in response"))
}

// ── Response types (only what we need to read) ─────────────────────────────

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}
