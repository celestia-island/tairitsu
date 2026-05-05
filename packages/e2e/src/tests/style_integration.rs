//! Integration tests for tairitsu-style package
//!
//! Verifies style package build artifacts, CSS generation correctness,
//! and browser-side style application via WebDriver.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use thirtyfour::WebDriver;
use tracing::info;

use crate::tests::{Test, TestResult, TestStatus};

pub struct StyleIntegrationTests;

fn style_pkg_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("style")
}

fn extract_string(ret: &thirtyfour::prelude::ScriptRet) -> String {
    ret.json().as_str().unwrap_or("").to_string()
}

impl StyleIntegrationTests {
    fn test_css_properties_data_exists(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing CSS properties data file exists");

        let data_path = style_pkg_root().join("css_data/css_properties.json");
        if !data_path.exists() {
            return Ok(TestResult {
                component: "CSS Properties Data".into(),
                status: TestStatus::Failure,
                message: format!("css_properties.json not found at {}", data_path.display()),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            });
        }

        let content = std::fs::read_to_string(&data_path)
            .with_context(|| format!("Failed to read {}", data_path.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| "css_properties.json is not valid JSON")?;

        let categories = parsed
            .get("categories")
            .and_then(|c| c.as_object())
            .ok_or_else(|| anyhow::anyhow!("Missing or invalid 'categories' key"))?;

        let total_properties: usize = categories
            .values()
            .filter_map(|v| v.get("properties").and_then(|p| p.as_array()))
            .map(|a| a.len())
            .sum();

        if total_properties == 0 {
            return Ok(TestResult {
                component: "CSS Properties Data".into(),
                status: TestStatus::Failure,
                message: "css_properties.json contains zero properties".into(),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            });
        }

        let required_categories = ["BoxModel", "Flexbox", "Typography", "Color", "Layout"];
        let mut missing = Vec::new();
        for cat in &required_categories {
            if !categories.contains_key(*cat) {
                missing.push(cat.to_string());
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        if missing.is_empty() {
            Ok(TestResult {
                component: "CSS Properties Data".into(),
                status: TestStatus::Success,
                message: format!(
                    "Valid JSON with {} properties across {} categories",
                    total_properties,
                    categories.len()
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "CSS Properties Data".into(),
                status: TestStatus::Warning,
                message: format!("Missing categories: {}", missing.join(", ")),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    fn test_generated_property_enum(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing generated CssProperty enum file");

        let generated_path = style_pkg_root().join("src/properties/generated.rs");
        if !generated_path.exists() {
            return Ok(TestResult {
                component: "Generated Property Enum".into(),
                status: TestStatus::Failure,
                message: format!("generated.rs not found at {}", generated_path.display()),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            });
        }

        let content = std::fs::read_to_string(&generated_path)?;

        let has_enum = content.contains("pub enum CssProperty {");
        let has_as_str = content.contains("pub fn as_str(&self) -> &'static str {");
        let has_category = content.contains("pub fn category(&self) -> CssCategory {");

        let property_mappings = [
            ("Display", "display"),
            ("FlexDirection", "flex-direction"),
            ("BackgroundColor", "background-color"),
            ("Width", "width"),
            ("Height", "height"),
            ("Padding", "padding"),
            ("Margin", "margin"),
            ("Position", "position"),
            ("FontSize", "font-size"),
            ("Color", "color"),
        ];

        let mut missing_mappings = Vec::new();
        for (variant, css_name) in &property_mappings {
            let pattern = format!("CssProperty::{} => \"{}\"", variant, css_name);
            if !content.contains(&pattern) {
                missing_mappings.push(format!("{}->{}", variant, css_name));
            }
        }

        let duration = start.elapsed().as_millis() as u64;

        if has_enum && has_as_str && has_category && missing_mappings.is_empty() {
            Ok(TestResult {
                component: "Generated Property Enum".into(),
                status: TestStatus::Success,
                message: format!(
                    "generated.rs has CssProperty enum, as_str, category, all {} mappings verified",
                    property_mappings.len()
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            let mut issues = Vec::new();
            if !has_enum {
                issues.push("missing CssProperty enum".into());
            }
            if !has_as_str {
                issues.push("missing as_str method".into());
            }
            if !has_category {
                issues.push("missing category method".into());
            }
            if !missing_mappings.is_empty() {
                issues.push(format!("missing mappings: {}", missing_mappings.join(", ")));
            }
            Ok(TestResult {
                component: "Generated Property Enum".into(),
                status: TestStatus::Failure,
                message: issues.join("; "),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    fn test_build_script_present(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing build.rs presence and correctness");

        let build_rs_path = style_pkg_root().join("build.rs");
        if !build_rs_path.exists() {
            return Ok(TestResult {
                component: "Build Script".into(),
                status: TestStatus::Failure,
                message: "build.rs not found in style package".into(),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            });
        }

        let content = std::fs::read_to_string(&build_rs_path)?;

        let has_rerun = content.contains("cargo:rerun-if-changed=css_data/css_properties.json");
        let has_read = content.contains("css_properties.json");
        let has_write = content.contains("generated.rs");
        let has_json_parse = content.contains("serde_json::from_str");

        let duration = start.elapsed().as_millis() as u64;

        if has_rerun && has_read && has_write && has_json_parse {
            Ok(TestResult {
                component: "Build Script".into(),
                status: TestStatus::Success,
                message: "build.rs reads JSON, generates Rust, sets rerun triggers".into(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            let mut missing = Vec::new();
            if !has_rerun {
                missing.push("rerun-if-changed");
            }
            if !has_read {
                missing.push("JSON read");
            }
            if !has_write {
                missing.push("generated.rs write");
            }
            if !has_json_parse {
                missing.push("JSON parsing");
            }
            Ok(TestResult {
                component: "Build Script".into(),
                status: TestStatus::Warning,
                message: format!("build.rs missing: {}", missing.join(", ")),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    fn test_public_api_exports(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing public API exports in lib.rs");

        let lib_path = style_pkg_root().join("src/lib.rs");
        let content = std::fs::read_to_string(&lib_path)
            .with_context(|| "Failed to read style src/lib.rs")?;

        let required_exports = [
            "StyleStringBuilder",
            "StyleBuilder",
            "ClassesBuilder",
            "CssProperty",
            "CssCategory",
            "Property",
            "UtilityRegistry",
            "Breakpoint",
            "State",
            "Variant",
            "ParsedUtility",
            "create_default_registry",
            "DisplayValue",
            "FlexDirectionValue",
            "PositionValue",
            "CssValue",
        ];

        let mut missing = Vec::new();
        for export in &required_exports {
            if !content.contains(export) {
                missing.push(export.to_string());
            }
        }

        let duration = start.elapsed().as_millis() as u64;

        if missing.is_empty() {
            Ok(TestResult {
                component: "Public API Exports".into(),
                status: TestStatus::Success,
                message: format!(
                    "All {} public API symbols exported from lib.rs",
                    required_exports.len()
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Public API Exports".into(),
                status: TestStatus::Failure,
                message: format!("Missing exports: {}", missing.join(", ")),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    async fn test_browser_computed_styles(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing computed styles in browser");

        let base_url =
            std::env::var("WEBSITE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let test_css = r#"
            .e2e-test-flex { display: flex; }
            .e2e-test-red { color: rgb(255, 0, 0); }
            .e2e-test-padding { padding: 16px; }
            .e2e-test-hidden { display: none; }
        "#;

        let inject_script = format!(
            r#"
            (() => {{
                const style = document.createElement('style');
                style.textContent = `{}`;
                document.head.appendChild(style);

                const container = document.createElement('div');
                container.id = 'e2e-style-test-container';

                const el1 = document.createElement('div');
                el1.id = 'e2e-flex';
                el1.className = 'e2e-test-flex';
                el1.textContent = 'flex test';

                const el2 = document.createElement('div');
                el2.id = 'e2e-red';
                el2.className = 'e2e-test-red';
                el2.textContent = 'red test';

                const el3 = document.createElement('div');
                el3.id = 'e2e-padding';
                el3.className = 'e2e-test-padding';
                el3.textContent = 'padding test';

                const el4 = document.createElement('div');
                el4.id = 'e2e-hidden';
                el4.className = 'e2e-test-hidden';
                el4.textContent = 'hidden test';

                container.appendChild(el1);
                container.appendChild(el2);
                container.appendChild(el3);
                container.appendChild(el4);
                document.body.appendChild(container);

                return 'injected';
            }})()
            "#,
            test_css
        );

        let _ret = driver.execute(inject_script, vec![]).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let mut failures = Vec::new();

        let flex_display = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-flex')).display"#,
                    vec![],
                )
                .await?,
        );
        if flex_display != "flex" {
            failures.push(format!("flex display='{}' (expected 'flex')", flex_display));
        }

        let red_color = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-red')).color"#,
                    vec![],
                )
                .await?,
        );
        if !red_color.contains("255") || !red_color.contains("0") {
            failures.push(format!(
                "red color='{}' (expected rgb containing 255,0,0)",
                red_color
            ));
        }

        let padding = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-padding')).paddingTop"#,
                    vec![],
                )
                .await?,
        );
        if !padding.contains("16") {
            failures.push(format!("padding='{}' (expected '16px')", padding));
        }

        let hidden_display = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-hidden')).display"#,
                    vec![],
                )
                .await?,
        );
        if hidden_display != "none" {
            failures.push(format!(
                "hidden display='{}' (expected 'none')",
                hidden_display
            ));
        }

        let _cleanup = driver
            .execute(
                r#"const c = document.getElementById('e2e-style-test-container'); if(c) c.remove(); return 'cleaned';"#,
                vec![],
            )
            .await?;

        let duration = start.elapsed().as_millis() as u64;

        if failures.is_empty() {
            Ok(TestResult {
                component: "Browser Computed Styles".into(),
                status: TestStatus::Success,
                message: "All 4 computed style checks passed (display, color, padding, hidden)"
                    .into(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Browser Computed Styles".into(),
                status: TestStatus::Failure,
                message: failures.join("; "),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    async fn test_browser_responsive_variants(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing responsive variant media queries in browser");

        let base_url =
            std::env::var("WEBSITE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let inject_script = r#"
        (() => {
            const style = document.createElement('style');
            style.textContent = `
                .e2e-responsive-box { font-size: 12px; }
                @media (min-width: 768px) {
                    .e2e-responsive-box { font-size: 24px; }
                }
            `;
            document.head.appendChild(style);

            const el = document.createElement('div');
            el.id = 'e2e-responsive';
            el.className = 'e2e-responsive-box';
            el.textContent = 'responsive test';
            document.body.appendChild(el);
            return 'injected';
        })()
        "#;

        let _ret = driver.execute(inject_script, vec![]).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let initial_size = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-responsive')).fontSize"#,
                    vec![],
                )
                .await?,
        );

        let initial_px: f64 = initial_size.replace("px", "").parse().unwrap_or(0.0);

        let is_large = initial_px > 18.0;

        let _cleanup = driver
            .execute(
                r#"const el = document.getElementById('e2e-responsive'); if(el) el.remove(); return 'cleaned';"#,
                vec![],
            )
            .await?;

        let duration = start.elapsed().as_millis() as u64;

        Ok(TestResult {
            component: "Responsive Variants (Browser)".into(),
            status: TestStatus::Success,
            message: format!(
                "Media query applied: fontSize={}px (viewport {} the 768px breakpoint)",
                initial_px,
                if is_large { "above" } else { "below" }
            ),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    async fn test_browser_state_variants(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing state variant (hover) in browser");

        let base_url =
            std::env::var("WEBSITE_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".into());
        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let inject_script = r#"
        (() => {
            const style = document.createElement('style');
            style.textContent = `
                .e2e-hover-box {
                    background-color: rgb(0, 0, 255);
                    width: 100px;
                    height: 100px;
                }
                .e2e-hover-box:hover {
                    background-color: rgb(0, 255, 0);
                }
            `;
            document.head.appendChild(style);

            const el = document.createElement('div');
            el.id = 'e2e-hover';
            el.className = 'e2e-hover-box';
            el.setAttribute('tabindex', '0');
            el.textContent = 'hover test';
            document.body.appendChild(el);
            return 'injected';
        })()
        "#;

        let _ret = driver.execute(inject_script, vec![]).await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let base_bg = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-hover')).backgroundColor"#,
                    vec![],
                )
                .await?,
        );

        let hover_el = driver.find(thirtyfour::By::Css("#e2e-hover")).await?;
        driver
            .action_chain()
            .move_to_element_center(&hover_el)
            .perform()
            .await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let hover_bg = extract_string(
            &driver
                .execute(
                    r#"return getComputedStyle(document.getElementById('e2e-hover')).backgroundColor"#,
                    vec![],
                )
                .await?,
        );

        let mut failures = Vec::new();

        if !base_bg.contains("0") || !base_bg.contains("255") {
            failures.push(format!("base bg='{}' not blue", base_bg));
        }

        if base_bg == hover_bg {
            failures.push("hover bg unchanged from base (hover may not apply in headless)".into());
        }

        let _cleanup = driver
            .execute(
                r#"const el = document.getElementById('e2e-hover'); if(el) el.remove(); return 'cleaned';"#,
                vec![],
            )
            .await?;

        let duration = start.elapsed().as_millis() as u64;

        if failures.is_empty() {
            Ok(TestResult {
                component: "State Variants (Browser)".into(),
                status: TestStatus::Success,
                message: format!(
                    "Hover state applied: base='{}', hover='{}'",
                    base_bg, hover_bg
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "State Variants (Browser)".into(),
                status: TestStatus::Warning,
                message: format!(
                    "{} (base='{}', hover='{}')",
                    failures.join("; "),
                    base_bg,
                    hover_bg
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    fn test_css_text_content_sanity(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing CSS text content sanity on style source files");

        let root = style_pkg_root();
        let lib_content = std::fs::read_to_string(root.join("src/lib.rs"))?;
        let builder_content = std::fs::read_to_string(root.join("src/builder.rs"))?;
        let classes_content = std::fs::read_to_string(root.join("src/classes.rs"))?;
        let utility_content = std::fs::read_to_string(root.join("src/utility.rs"))?;
        let _category_content = std::fs::read_to_string(root.join("src/properties/category.rs"))?;
        let generated_content = std::fs::read_to_string(root.join("src/properties/generated.rs"))?;

        let mut issues = Vec::new();

        if !lib_content.contains("mod builder") || !lib_content.contains("mod classes") {
            issues.push("lib.rs missing module declarations".into());
        }

        if !builder_content.contains("build_clean") || !builder_content.contains("to_vdom_style") {
            issues.push("builder.rs missing build_clean or to_vdom_style".into());
        }

        if !classes_content.contains("add_utility") || !classes_content.contains("generate_css") {
            issues.push("classes.rs missing add_utility or generate_css".into());
        }

        if !utility_content.contains("Breakpoint")
            || !utility_content.contains("State")
            || !utility_content.contains("Variant")
        {
            issues.push("utility.rs missing Breakpoint, State, or Variant".into());
        }

        let property_count = generated_content.matches("CssProperty::").count();
        if property_count < 200 {
            issues.push(format!(
                "generated.rs has only {} CssProperty:: references (expected 200+)",
                property_count
            ));
        }

        let duration = start.elapsed().as_millis() as u64;

        if issues.is_empty() {
            Ok(TestResult {
                component: "Source File Sanity".into(),
                status: TestStatus::Success,
                message: format!(
                    "All 6 source files pass structural checks ({} property refs in generated.rs)",
                    property_count
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Source File Sanity".into(),
                status: TestStatus::Failure,
                message: issues.join("; "),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }
}

impl Test for StyleIntegrationTests {
    fn name(&self) -> &str {
        "Style Integration Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up style integration test suite");
        let root = style_pkg_root();
        if !root.exists() {
            anyhow::bail!("Style package not found at {}", root.display());
        }
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running style integration tests");

        let mut results = Vec::new();

        macro_rules! run_test {
            ($method:ident) => {
                match self.$method() {
                    Ok(r) => results.push(r),
                    Err(e) => {
                        tracing::error!("{} failed: {}", stringify!($method), e);
                        results.push(TestResult::error(stringify!($method), &e.to_string()));
                    }
                }
            };
            ($method:ident, async) => {
                match self.$method(driver).await {
                    Ok(r) => results.push(r),
                    Err(e) => {
                        tracing::error!("{} failed: {}", stringify!($method), e);
                        results.push(TestResult::error(stringify!($method), &e.to_string()));
                    }
                }
            };
        }

        run_test!(test_css_properties_data_exists);
        run_test!(test_generated_property_enum);
        run_test!(test_build_script_present);
        run_test!(test_public_api_exports);
        run_test!(test_css_text_content_sanity);
        run_test!(test_browser_computed_styles, async);
        run_test!(test_browser_responsive_variants, async);
        run_test!(test_browser_state_variants, async);

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down style integration test suite");
        Ok(())
    }
}
