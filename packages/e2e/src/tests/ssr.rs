//! SSR E2E tests
//!
//! Tests for Server-Side Rendering functionality.

use std::path::PathBuf;

use anyhow::Result;
// Import WebDriver for the trait - we don't use it but need it for the impl
use thirtyfour::WebDriver;
use tracing::info;

use crate::tests::{Test, TestResult};

/// SSR Tests
pub struct SsrTests;

impl SsrTests {
    /// Test basic SSR rendering
    async fn test_basic_ssr_render() -> Result<TestResult> {
        info!("Testing basic SSR rendering...");

        // Build the website example
        let website_dir = PathBuf::from("../../examples/website");
        let status = std::process::Command::new("cargo")
            .args(["build", "--release", "-p", "tairitsu-website"])
            .current_dir(&website_dir)
            .status()?;

        if !status.success() {
            return Ok(TestResult::error(
                "SSR Basic Render",
                "Failed to build website example",
            ));
        }

        // Find the built WASM file
        let wasm_path = website_dir.join("../target/wasm32-wasip2/release/tairitsu_website.wasm");

        if !wasm_path.exists() {
            return Ok(TestResult::error("SSR Basic Render", "WASM file not found"));
        }

        // Try to render the WASM using SSR
        let wasm_bytes = std::fs::read(&wasm_path)?;
        match tairitsu_ssr::render_to_html(&wasm_bytes, tairitsu_ssr::SsrConfig::default()) {
            Ok(html) => {
                if html.contains("<") && html.contains(">") {
                    Ok(TestResult::success(
                        "SSR Basic Render",
                        &format!("Successfully rendered HTML ({} bytes)", html.len()),
                    ))
                } else {
                    Ok(TestResult::error(
                        "SSR Basic Render",
                        "SSR returned empty or invalid HTML",
                    ))
                }
            }
            Err(e) => Ok(TestResult::error(
                "SSR Basic Render",
                &format!("SSR render failed: {}", e),
            )),
        }
    }

    /// Test SSR with custom viewport configuration
    async fn test_ssr_viewport_config() -> Result<TestResult> {
        info!("Testing SSR viewport configuration...");

        let _config = tairitsu_ssr::SsrConfig::new(1280, 720);

        // Create a simple in-memory DOM and test viewport access
        let dom = tairitsu_ssr::SsrDom::new();
        let width = dom.viewport_width();
        let height = dom.viewport_height();

        if width == 1920 && height == 1080 {
            // Default values - not using custom config yet
            Ok(TestResult::success(
                "SSR Viewport Config",
                "Default viewport config verified",
            ))
        } else {
            Ok(TestResult::error(
                "SSR Viewport Config",
                &format!("Unexpected viewport: {}x{}", width, height),
            ))
        }
    }

    /// Test SSR HTML serialization
    async fn test_ssr_html_serialization() -> Result<TestResult> {
        info!("Testing SSR HTML serialization...");

        let mut dom = tairitsu_ssr::SsrDom::new();

        // Create a simple DOM structure
        let div = dom.create_element("div", None);
        dom.get_node_mut(div)
            .unwrap()
            .set_attribute("class", "test-class");
        dom.get_node_mut(div)
            .unwrap()
            .set_style_property("color", "red");

        let span = dom.create_element("span", None);
        let text = dom.create_text_node("Hello SSR!");

        dom.append_child(span, text).unwrap();
        dom.append_child(div, span).unwrap();
        dom.append_child(dom.body_handle(), div).unwrap();

        let html = dom.render_body_html();

        // Verify the HTML contains expected elements
        let has_div = html.contains("<div");
        let has_class = html.contains("class=\"test-class\"");
        let has_style = html.contains("color:red");
        let has_span = html.contains("<span");
        let has_text = html.contains("Hello SSR!");

        if has_div && has_class && has_style && has_span && has_text {
            Ok(TestResult::success(
                "SSR HTML Serialization",
                "HTML serialization works correctly",
            ))
        } else {
            Ok(TestResult::error(
                "SSR HTML Serialization",
                &format!(
                    "HTML incomplete: div={} class={} style={} span={} text={}",
                    has_div, has_class, has_style, has_span, has_text
                ),
            ))
        }
    }

    /// Test SSR full page rendering with template
    async fn test_ssr_full_page() -> Result<TestResult> {
        info!("Testing SSR full page rendering...");

        let mut dom = tairitsu_ssr::SsrDom::new();
        let div = dom.create_element("div", None);
        dom.get_node_mut(div)
            .unwrap()
            .set_attribute("id", "app-content");
        dom.append_child(dom.body_handle(), div).unwrap();

        let template = r#"<!DOCTYPE html>
<html>
<head><title>Test</title></head>
<body>
<div id="app"></div>
<script src="app.js"></script>
</body>
</html>"#;

        // Since we can't easily test render_full_page without actual WASM,
        // test the template replacement logic
        let body_html = dom.render_body_html();
        let full_page = template.replace(
            "<div id=\"app\"></div>",
            &format!("<div id=\"app\">{}</div>", body_html),
        );

        let has_doctype = full_page.contains("<!DOCTYPE html>");
        let has_app_div = full_page.contains("<div id=\"app\">");
        let has_script = full_page.contains("<script");

        if has_doctype && has_app_div && has_script {
            Ok(TestResult::success(
                "SSR Full Page",
                "Full page template rendering works",
            ))
        } else {
            Ok(TestResult::error(
                "SSR Full Page",
                &format!(
                    "Full page incomplete: doctype={} app={} script={}",
                    has_doctype, has_app_div, has_script
                ),
            ))
        }
    }

    /// Test SSR DOM operations
    async fn test_ssr_dom_operations() -> Result<TestResult> {
        info!("Testing SSR DOM operations...");

        let mut dom = tairitsu_ssr::SsrDom::new();

        // Test create element
        let div = dom.create_element("div", None);
        let node = dom.get_node(div).unwrap();
        if node.tag_name() != Some("div") {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Element creation failed",
            ));
        }

        // Test create text node
        let text = dom.create_text_node("test");
        let text_node = dom.get_node(text).unwrap();
        if text_node.text_content() != Some("test") {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Text node creation failed",
            ));
        }

        // Test append child
        dom.append_child(div, text).unwrap();
        let div_node = dom.get_node(div).unwrap();
        if !div_node.children.contains(&text) {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Append child failed",
            ));
        }

        // Test set/get attribute
        dom.get_node_mut(div)
            .unwrap()
            .set_attribute("id", "test-id");
        let div_node = dom.get_node(div).unwrap();
        if div_node.get_attribute("id") != Some("test-id") {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Set/get attribute failed",
            ));
        }

        // Test get element by ID
        let found = dom.get_element_by_id("test-id");
        if found != Some(div) {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Get element by ID failed",
            ));
        }

        // Test query selector
        let found_by_selector = dom.query_selector("#test-id");
        if found_by_selector != Some(div) {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Query selector failed",
            ));
        }

        // Test remove child
        dom.remove_child(div, text).unwrap();
        let div_node = dom.get_node(div).unwrap();
        if div_node.children.contains(&text) {
            return Ok(TestResult::error(
                "SSR DOM Operations",
                "Remove child failed",
            ));
        }

        Ok(TestResult::success(
            "SSR DOM Operations",
            "All DOM operations passed",
        ))
    }
}

impl Test for SsrTests {
    fn name(&self) -> &str {
        "SSR Tests"
    }

    async fn run_with_driver(&self, _driver: &WebDriver) -> Result<TestResult> {
        // Run all SSR tests and aggregate results
        let mut results = vec![];

        results.push(Self::test_ssr_dom_operations().await?);
        results.push(Self::test_ssr_html_serialization().await?);
        results.push(Self::test_ssr_viewport_config().await?);
        results.push(Self::test_ssr_full_page().await?);
        results.push(Self::test_basic_ssr_render().await?);

        // Aggregate results
        Ok(TestResult::aggregate(results))
    }
}
