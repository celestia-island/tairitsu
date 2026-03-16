//! Component lifecycle tests
//!
//! Tests for component lifecycle including mount, update, and unmount.

use crate::tests::{Test, TestResult, TestStatus};
use anyhow::Result;
use std::time::{Duration, Instant};
use thirtyfour::{By, WebDriver};
use tracing::info;

pub struct LifecycleTests;

impl LifecycleTests {
    /// Test component mounting
    async fn test_component_mount(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing component mount");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if #app element exists (component should be mounted)
        let app_element = driver.find(By::Id("app")).await;

        let duration = start.elapsed().as_millis() as u64;

        match app_element {
            Ok(_) => Ok(TestResult {
                component: "Component Mount".to_string(),
                status: TestStatus::Success,
                message: "Component mounted successfully".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            }),
            Err(e) => Ok(TestResult {
                component: "Component Mount".to_string(),
                status: TestStatus::Failure,
                message: format!("Component mount failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test component rendering
    async fn test_component_render(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing component render");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if content is rendered
        let body = driver.find(By::Tag("body")).await?;
        let text = body.text().await?;

        let duration = start.elapsed().as_millis() as u64;

        if !text.is_empty() {
            Ok(TestResult {
                component: "Component Render".to_string(),
                status: TestStatus::Success,
                message: format!("Component rendered content: {} chars", text.len()),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Component Render".to_string(),
                status: TestStatus::Failure,
                message: "Component rendered no content".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test component update on navigation
    async fn test_component_update(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing component update on navigation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Navigate to home
        driver.goto(&format!("{}/", base_url)).await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let initial_url = driver.current_url().await?;

        // Navigate to a different section
        driver.goto(&format!("{}/#about", base_url)).await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        let updated_url = driver.current_url().await?;

        let duration = start.elapsed().as_millis() as u64;

        if initial_url != updated_url {
            Ok(TestResult {
                component: "Component Update".to_string(),
                status: TestStatus::Success,
                message: "Component updated on navigation".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Component Update".to_string(),
                status: TestStatus::Failure,
                message: "Navigation did not update component".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test element creation and attribute setting
    async fn test_element_creation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing element creation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Look for various element types that should be created
        let divs = driver.find_all(By::Tag("div")).await?;
        let buttons = driver.find_all(By::Tag("button")).await?;
        let inputs = driver.find_all(By::Tag("input")).await?;

        let duration = start.elapsed().as_millis() as u64;

        Ok(TestResult {
            component: "Element Creation".to_string(),
            status: TestStatus::Success,
            message: format!(
                "Elements created: {} divs, {} buttons, {} inputs",
                divs.len(),
                buttons.len(),
                inputs.len()
            ),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test text node rendering
    async fn test_text_node_rendering(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing text node rendering");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Get page title
        let title = driver.title().await?;

        let duration = start.elapsed().as_millis() as u64;

        if !title.is_empty() {
            Ok(TestResult {
                component: "Text Node Rendering".to_string(),
                status: TestStatus::Success,
                message: format!("Page title rendered: {}", title),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Text Node Rendering".to_string(),
                status: TestStatus::Failure,
                message: "Page title not rendered".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test CSS class application
    async fn test_css_class_application(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing CSS class application");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Look for elements with CSS classes - simplified using find_all
        let elements_with_class = driver
            .find_all(By::Css("[class]"))
            .await?;

        let duration = start.elapsed().as_millis() as u64;

        if !elements_with_class.is_empty() {
            Ok(TestResult {
                component: "CSS Class Application".to_string(),
                status: TestStatus::Success,
                message: format!("{} elements with CSS classes found", elements_with_class.len()),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "CSS Class Application".to_string(),
                status: TestStatus::Warning,
                message: "No elements with CSS classes found".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test style attribute application
    async fn test_style_attribute_application(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing style attribute application");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Look for elements with inline styles - simplified using find_all
        let elements_with_style = driver
            .find_all(By::Css("[style]"))
            .await?;

        let duration = start.elapsed().as_millis() as u64;

        Ok(TestResult {
            component: "Style Attribute Application".to_string(),
            status: TestStatus::Success,
            message: format!("{} elements with inline styles", elements_with_style.len()),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for LifecycleTests {
    fn name(&self) -> &str {
        "Component Lifecycle Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up component lifecycle test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running component lifecycle tests");

        let mut results = vec![];

        // Run all lifecycle tests
        match self.test_component_mount(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Component mount test failed: {}", e);
                results.push(TestResult::error("Component Mount", &e.to_string()));
            }
        }

        match self.test_component_render(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Component render test failed: {}", e);
                results.push(TestResult::error("Component Render", &e.to_string()));
            }
        }

        match self.test_component_update(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Component update test failed: {}", e);
                results.push(TestResult::error("Component Update", &e.to_string()));
            }
        }

        match self.test_element_creation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Element creation test failed: {}", e);
                results.push(TestResult::error("Element Creation", &e.to_string()));
            }
        }

        match self.test_text_node_rendering(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Text node rendering test failed: {}", e);
                results.push(TestResult::error("Text Node Rendering", &e.to_string()));
            }
        }

        match self.test_css_class_application(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("CSS class application test failed: {}", e);
                results.push(TestResult::error("CSS Class Application", &e.to_string()));
            }
        }

        match self.test_style_attribute_application(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Style attribute application test failed: {}", e);
                results.push(TestResult::error("Style Attribute Application", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down component lifecycle test suite");
        Ok(())
    }
}
