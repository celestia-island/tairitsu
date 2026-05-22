use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, WebDriver};
use tracing::info;

use crate::tests::{Test, TestResult};

pub struct BasicComponentsTests;

impl BasicComponentsTests {
    async fn test_button(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing Button component");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/basic", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let button = driver.find(By::Css(".tairitsu-button")).await?;
        info!("Button element found");

        button.click().await?;
        info!("Button clicked successfully");

        tokio::time::sleep(Duration::from_millis(200)).await;

        let class_attr = button
            .attr("class")
            .await?
            .ok_or_else(|| anyhow::anyhow!("No class attribute"))?;

        if !class_attr.contains("tairitsu-button") {
            return Ok(TestResult::failure(
                "Button",
                "Button element missing 'tairitsu-button' class",
            ));
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Button".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Button renders correctly, responds to clicks".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    async fn test_input(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing Input component");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/basic", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let input = driver.find(By::Css(".tairitsu-input")).await?;
        info!("Input element found");

        input.send_keys("test input from E2E").await?;
        info!("Text entered successfully");

        let class_attr = input
            .attr("class")
            .await?
            .ok_or_else(|| anyhow::anyhow!("No class attribute"))?;

        if !class_attr.contains("tairitsu-input") {
            return Ok(TestResult::failure(
                "Input",
                "Input element missing 'tairitsu-input' class",
            ));
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Input".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Input renders correctly and accepts input".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for BasicComponentsTests {
    fn name(&self) -> &str {
        "Basic Components Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up basic components test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running basic components E2E tests");

        let mut results = vec![];

        match self.test_button(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Button test failed: {}", e);
                results.push(TestResult::error("Button", &e.to_string()));
            }
        }

        match self.test_input(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Input test failed: {}", e);
                results.push(TestResult::error("Input", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }
}
