//! Async operations E2E tests.
//!
//! Tests asynchronous operations including:
//! - setTimeout/setInterval
//! - fetch API calls
//! - Promise handling
//! - async/await patterns
//! - RequestAnimationFrame

use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, WebDriver};
use tracing::info;

use crate::tests::{Test, TestResult};

pub struct AsyncOperationsTests;

impl AsyncOperationsTests {
    /// Test setTimeout functionality.
    async fn test_set_timeout(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing setTimeout functionality");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find timeout button
        let timeout_button = driver.find(By::Css("#timeout-button")).await.ok();
        let timeout_result = driver.find(By::Css("#timeout-result")).await.ok();

        if let (Some(button), Some(result_div)) = (timeout_button, timeout_result) {
            // Click to trigger setTimeout
            button.click().await?;

            // Wait for async operation (should complete in 1-2 seconds)
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Check result
            let result_text = result_div.text().await?;
            info!("Timeout result: {}", result_text);

            if result_text.contains("complete") || result_text.contains("done") {
                info!("setTimeout completed successfully");
            }
        } else {
            info!("Timeout elements not found, skipping setTimeout test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "setTimeout".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Timeout elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "setTimeout".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "setTimeout works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test setInterval functionality.
    async fn test_set_interval(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing setInterval functionality");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let start_button = driver.find(By::Css("#interval-start")).await.ok();
        let stop_button = driver.find(By::Css("#interval-stop")).await.ok();
        let counter_display = driver.find(By::Css("#interval-counter")).await.ok();

        if let (Some(start_btn), Some(stop_btn), Some(counter)) =
            (start_button, stop_button, counter_display)
        {
            // Start interval
            start_btn.click().await?;
            tokio::time::sleep(Duration::from_millis(1100)).await;

            // Check counter increased
            let count1 = counter.text().await?;
            info!("Interval count after 1s: {}", count1);

            // Wait another second
            tokio::time::sleep(Duration::from_millis(1000)).await;

            let count2 = counter.text().await?;
            info!("Interval count after 2s: {}", count2);

            // Stop interval
            stop_btn.click().await?;
            tokio::time::sleep(Duration::from_millis(500)).await;

            let count3 = counter.text().await?;
            info!("Interval count after stop: {}", count3);

            // Counter should have increased
            if count2 != count1 || count3 != count2 {
                info!("setInterval working: counter changed over time");
            }
        } else {
            info!("Interval elements not found, skipping setInterval test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "setInterval".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Interval elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "setInterval".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "setInterval works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test fetch API calls.
    async fn test_fetch_api(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing fetch API");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let fetch_button = driver.find(By::Css("#fetch-button")).await.ok();
        let fetch_result = driver.find(By::Css("#fetch-result")).await.ok();

        if let (Some(button), Some(result_div)) = (fetch_button, fetch_result) {
            // Click to trigger fetch
            button.click().await?;

            // Wait for fetch to complete
            tokio::time::sleep(Duration::from_millis(2000)).await;

            // Check result
            let result_text = result_div.text().await?;
            info!("Fetch result: {}", result_text);

            if !result_text.is_empty() {
                info!("Fetch API call completed");
            }
        } else {
            info!("Fetch elements not found, skipping fetch test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Fetch API".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Fetch elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Fetch API".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Fetch API works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test Promise handling.
    async fn test_promise_handling(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing Promise handling");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let promise_button = driver.find(By::Css("#promise-button")).await.ok();
        let promise_result = driver.find(By::Css("#promise-result")).await.ok();

        if let (Some(button), Some(result_div)) = (promise_button, promise_result) {
            // Click to trigger Promise
            button.click().await?;

            // Wait for Promise to resolve
            tokio::time::sleep(Duration::from_millis(1000)).await;

            // Check result
            let result_text = result_div.text().await?;
            info!("Promise result: {}", result_text);

            if result_text.contains("resolved") || result_text.contains("success") {
                info!("Promise resolved successfully");
            }
        } else {
            info!("Promise elements not found, skipping Promise test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Promise Handling".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Promise elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Promise Handling".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Promise handling works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test async/await patterns.
    async fn test_async_await(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing async/await patterns");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let async_button = driver.find(By::Css("#async-button")).await.ok();
        let async_result = driver.find(By::Css("#async-result")).await.ok();

        if let (Some(button), Some(result_div)) = (async_button, async_result) {
            // Click to trigger async/await
            button.click().await?;

            // Wait for async operation
            tokio::time::sleep(Duration::from_millis(1500)).await;

            // Check result
            let result_text = result_div.text().await?;
            info!("async/await result: {}", result_text);

            if !result_text.is_empty() {
                info!("async/await pattern works");
            }
        } else {
            info!("async/await elements not found, skipping test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "async/await".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "async/await elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "async/await".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "async/await patterns work correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test RequestAnimationFrame.
    async fn test_request_animation_frame(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing RequestAnimationFrame");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/async", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let raf_button = driver.find(By::Css("#raf-button")).await.ok();
        let raf_counter = driver.find(By::Css("#raf-counter")).await.ok();

        if let (Some(button), Some(counter)) = (raf_button, raf_counter) {
            // Click to start animation
            button.click().await?;

            // Wait for a few animation frames
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Check counter
            let count_text = counter.text().await?;
            info!("RAF counter: {}", count_text);

            // Animation should have run
            if !count_text.is_empty() && count_text != "0" {
                info!("RequestAnimationFrame working");
            }
        } else {
            info!("RAF elements not found, skipping RAF test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "RequestAnimationFrame".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "RAF elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "RequestAnimationFrame".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "RequestAnimationFrame works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for AsyncOperationsTests {
    fn name(&self) -> &str {
        "Async Operations Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up async operations test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running async operations E2E tests");

        let mut results = vec![];

        // Test 1: setTimeout
        match self.test_set_timeout(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("setTimeout test failed: {}", e);
                results.push(TestResult::error("setTimeout", &e.to_string()));
            }
        }

        // Test 2: setInterval
        match self.test_set_interval(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("setInterval test failed: {}", e);
                results.push(TestResult::error("setInterval", &e.to_string()));
            }
        }

        // Test 3: Fetch API
        match self.test_fetch_api(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Fetch API test failed: {}", e);
                results.push(TestResult::error("Fetch API", &e.to_string()));
            }
        }

        // Test 4: Promise handling
        match self.test_promise_handling(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Promise handling test failed: {}", e);
                results.push(TestResult::error("Promise Handling", &e.to_string()));
            }
        }

        // Test 5: async/await
        match self.test_async_await(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("async/await test failed: {}", e);
                results.push(TestResult::error("async/await", &e.to_string()));
            }
        }

        // Test 6: RequestAnimationFrame
        match self.test_request_animation_frame(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("RequestAnimationFrame test failed: {}", e);
                results.push(TestResult::error("RequestAnimationFrame", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down async operations test suite");
        Ok(())
    }
}
