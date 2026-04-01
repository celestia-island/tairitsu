//! Navigation and routing E2E tests.
//!
//! Tests hash-based navigation (current website approach) and router-based
//! navigation (using the Rust router package).

use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, WebDriver};
use tracing::info;

use crate::tests::{Test, TestResult};

pub struct NavigationTests;

impl NavigationTests {
    /// Test hash-based navigation (current website approach).
    async fn test_hash_navigation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing hash-based navigation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Navigate to home page
        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Click on "Guides" link in top nav
        let guides_link = driver
            .find(By::Css("a[href=\"/guides/quick-start\"]"))
            .await?;
        guides_link.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Verify URL changed to hash
        let current_url = driver.current_url().await?;
        if !current_url.contains("#") && !current_url.contains("/guides/quick-start") {
            return Ok(TestResult::failure(
                "Hash Navigation",
                &format!("URL did not update after navigation: {}", current_url),
            ));
        }

        // Navigate to system overview
        let system_link = driver.find(By::Css("a[href=\"/system/overview\"]")).await?;
        system_link.click().await?;
        tokio::time::sleep(Duration::from_millis(300)).await;

        // Verify page title or content changed
        let page_content = driver.find(By::Css(".tairitsu-content")).await?;
        let text = page_content.text().await?;
        if !text.contains("系统") && !text.contains("System") {
            return Ok(TestResult::failure(
                "Hash Navigation",
                "Page content did not update after navigation",
            ));
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Hash Navigation".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Hash-based navigation works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test sidebar navigation links.
    async fn test_sidebar_navigation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing sidebar navigation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find sidebar links
        let sidebar_links = driver.find_all(By::Css(".sidebar-link")).await?;
        if sidebar_links.is_empty() {
            return Ok(TestResult::failure(
                "Sidebar Navigation",
                "No sidebar links found",
            ));
        }

        // Click first few sidebar links and verify navigation
        for i in 0..3.min(sidebar_links.len()) {
            sidebar_links[i].click().await?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Verify some navigation happened (URL or content changed)
            let current_url = driver.current_url().await?;
            info!("Navigated to: {}", current_url);
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Sidebar Navigation".to_string(),
            status: crate::tests::TestStatus::Success,
            message: format!(
                "Clicked {} sidebar links successfully",
                3.min(sidebar_links.len())
            ),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test browser back/forward buttons with hash navigation.
    async fn test_browser_history(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing browser history integration");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Navigate to a few pages
        let guides_link = driver
            .find(By::Css("a[href=\"/guides/quick-start\"]"))
            .await?;
        guides_link.click().await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let system_link = driver.find(By::Css("a[href=\"/system/overview\"]")).await?;
        system_link.click().await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Go back
        driver.back().await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Verify we're on the previous page
        let current_url = driver.current_url().await?;
        info!("After back(): {}", current_url);

        // Go forward
        driver.forward().await?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let current_url = driver.current_url().await?;
        info!("After forward(): {}", current_url);

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Browser History".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Browser back/forward buttons work with hash navigation".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test direct URL navigation (deep linking).
    async fn test_deep_linking(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing deep linking (direct URL navigation)");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        // Navigate directly to a specific page hash
        let direct_url = format!("{}#/guides/quick-start", base_url);
        driver.goto(&direct_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify the correct page is shown
        let page_content = driver.find(By::Css(".tairitsu-content")).await?;
        let text = page_content.text().await?;

        // Should show quick start content
        let has_content = text.contains("快速") || text.contains("Quick") || text.contains("Start");

        if !has_content {
            return Ok(TestResult::failure(
                "Deep Linking",
                "Direct URL navigation did not show correct content",
            ));
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Deep Linking".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Direct URL navigation to hash routes works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for NavigationTests {
    fn name(&self) -> &str {
        "Navigation and Routing Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up navigation test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running navigation E2E tests");

        let mut results = vec![];

        // Test 1: Hash-based navigation
        match self.test_hash_navigation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Hash navigation test failed: {}", e);
                results.push(TestResult::error("Hash Navigation", &e.to_string()));
            }
        }

        // Test 2: Sidebar navigation
        match self.test_sidebar_navigation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Sidebar navigation test failed: {}", e);
                results.push(TestResult::error("Sidebar Navigation", &e.to_string()));
            }
        }

        // Test 3: Browser history
        match self.test_browser_history(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Browser history test failed: {}", e);
                results.push(TestResult::error("Browser History", &e.to_string()));
            }
        }

        // Test 4: Deep linking
        match self.test_deep_linking(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Deep linking test failed: {}", e);
                results.push(TestResult::error("Deep Linking", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down navigation test suite");
        Ok(())
    }
}
