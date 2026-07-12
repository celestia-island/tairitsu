//! tairitsu-mcp binary — MCP server with vision analysis tools.
//!
//! Run as: `tairitsu-mcp` (stdio transport, for MCP client integration).
//! Requires a running shirabe debug server for screenshot capture.

use rmcp::{
    handler::server::wrapper::Parameters, model::*, service::RequestContext, tool, tool_handler,
    tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use serde::Deserialize;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("tairitsu-mcp starting (stdio transport)");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let server = VisionServer;
        let transport = rmcp::transport::stdio();
        let handle = server.serve(transport).await.expect("MCP server failed");
        handle.waiting().await.expect("MCP server crashed");
    });
}

#[derive(Debug, Deserialize, JsonSchema)]
struct AnalyzeArgs {
    /// Analysis prompt. Defaults to a rendering QA checklist.
    #[serde(default)]
    prompt: Option<String>,
    /// Shirabe debug server URL (default: http://127.0.0.1:3001).
    #[serde(default)]
    shirabe_url: Option<String>,
    /// Specific model ID to use (default: cheapest available with an API key).
    #[serde(default)]
    model: Option<String>,
    /// Capture full page instead of viewport only.
    #[serde(default)]
    full_page: Option<bool>,
}

struct VisionServer;

#[tool_router]
impl VisionServer {
    #[tool(
        name = "analyze_screenshot",
        description = "Capture a screenshot from a running shirabe browser instance and analyze it with a vision LLM. Returns a detailed analysis report covering layout, rendering issues, CSS problems, and improvement suggestions."
    )]
    async fn analyze_screenshot(
        &self,
        Parameters(args): Parameters<AnalyzeArgs>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match tairitsu_mcp::do_analyze(args.prompt, args.shirabe_url, args.model, args.full_page)
            .await
        {
            Ok(report) => Ok(CallToolResult::success(vec![Content::text(report)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Analysis failed: {e}"
            ))])),
        }
    }

    #[tool(
        name = "list_vision_models",
        description = "List all vision-capable LLM models from the provider-registry, sorted by price (cheapest first). Shows which ones have API keys configured."
    )]
    async fn list_vision_models(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match tairitsu_mcp::do_list_models() {
            Ok(models) => {
                let lines: Vec<String> = models
                    .iter()
                    .map(|m| {
                        let key_status = if m.has_key { "✓" } else { "✗" };
                        format!(
                            "  {} {:<30} {:<15} ${:>6.2}/M  key:{} ({})",
                            key_status,
                            m.model_id,
                            m.provider,
                            m.input_price,
                            key_status,
                            m.env_var
                        )
                    })
                    .collect();
                let summary = format!(
                    "Vision models ({} total, sorted by price):\n{}",
                    models.len(),
                    lines.join("\n")
                );
                Ok(CallToolResult::success(vec![Content::text(summary)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to load models: {e}"
            ))])),
        }
    }
}

#[tool_handler(router = VisionServer::tool_router())]
impl ServerHandler for VisionServer {}
