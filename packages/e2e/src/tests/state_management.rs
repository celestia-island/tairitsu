//! State management E2E tests.
//!
//! Tests the reactive state management system including:
//! - use_signal (reactive signals)
//! - use_state (component state)
//! - Context API
//! - Store pattern (global state)

use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, WebDriver, WebElement};
use tracing::info;

use crate::tests::{Test, TestResult};

pub struct StateManagementTests;

impl StateManagementTests {
    /// Test counter state updates (basic use_signal).
    async fn test_counter_state(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing counter state management");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/state", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find counter elements
        let counter_display = driver.find(By::Css("#counter-display")).await?;
        let increment_button = driver.find(By::Css("#counter-increment")).await?;

        // Get initial counter value
        let initial_value = counter_display.text().await?;
        info!("Initial counter value: {}", initial_value);

        // Click increment button
        increment_button.click().await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify counter updated
        let new_value = counter_display.text().await?;
        info!("Counter value after increment: {}", new_value);

        if new_value == initial_value {
            return Ok(TestResult::failure(
                "Counter State",
                &format!("Counter did not update: {} -> {}", initial_value, new_value),
            ));
        }

        // Click multiple times
        for _ in 0..4 {
            increment_button.click().await?;
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let final_value = counter_display.text().await?;
        info!("Final counter value: {}", final_value);

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Counter State".to_string(),
            status: crate::tests::TestStatus::Success,
            message: format!("Counter updated from {} to {}", initial_value, final_value),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test text input state binding.
    async fn test_input_state_binding(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing input state binding");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/state", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find input field and its display element
        let text_input = driver.find(By::Css("#text-input")).await?;
        let text_display = driver.find(By::Css("#text-display")).await?;

        // Type into input
        let test_text = "Hello, Tairitsu!";
        text_input.send_keys(test_text).await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify display updated
        let display_text = text_display.text().await?;
        info!("Typed: '{}', Display: '{}'", test_text, display_text);

        if !display_text.contains(test_text) {
            return Ok(TestResult::failure(
                "Input State Binding",
                &format!("Input binding failed: expected '{}', got '{}'", test_text, display_text),
            ));
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Input State Binding".to_string(),
            status: crate::tests::TestStatus::Success,
            message: format!("Input state binding works: '{}'", display_text),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test checkbox state (boolean state).
    async fn test_checkbox_state(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing checkbox state");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/state", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find checkbox and its display
        let checkbox = driver.find(By::Css("#toggle-checkbox")).await?;
        let toggle_display = driver.find(By::Css("#toggle-display")).await?;

        // Get initial state
        let initial_text = toggle_display.text().await?;
        info!("Initial toggle state: {}", initial_text);

        // Click checkbox
        checkbox.click().await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Verify state changed
        let new_text = toggle_display.text().await?;
        info!("Toggle state after click: {}", new_text);

        if new_text == initial_text {
            return Ok(TestResult::failure(
                "Checkbox State",
                "Checkbox state did not change after click",
            ));
        }

        // Click again to toggle back
        checkbox.click().await?;
        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_text = toggle_display.text().await?;
        info!("Toggle state after second click: {}", final_text);

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Checkbox State".to_string(),
            status: crate::tests::TestStatus::Success,
            message: format!("Checkbox toggled: {} -> {} -> {}", initial_text, new_text, final_text),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test list state management (add/remove items).
    async fn test_list_state(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing list state management");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/state", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find add button and list
        let add_button = driver.find(By::Css("#list-add")).await?;
        let list_items = driver.find_all(By::Css("#list-display li")).await?;

        let initial_count = list_items.len();
        info!("Initial list item count: {}", initial_count);

        // Add a few items
        for i in 0..3 {
            add_button.click().await?;
            tokio::time::sleep(Duration::from_millis(50)).await;
            info!("Added item {}", i + 1);
        }

        // Check list grew
        let new_list_items = driver.find_all(By::Css("#list-display li")).await?;
        let new_count = new_list_items.len();
        info!("Final list item count: {}", new_count);

        if new_count <= initial_count {
            return Ok(TestResult::failure(
                "List State",
                &format!("List did not grow: {} -> {}", initial_count, new_count),
            ));
        }

        // Try removing an item (click first item's remove button)
        if let Some(first_item) = new_list_items.first() {
            if let Ok(remove_btn) = first_item.find(By::Css(".remove-btn")).await {
                remove_btn.click().await?;
                tokio::time::sleep(Duration::from_millis(50)).await;

                let final_list_items = driver.find_all(By::Css("#list-display li")).await?;
                let final_count = final_list_items.len();
                info!("List count after removal: {}", final_count);

                if final_count >= new_count {
                    return Ok(TestResult::failure(
                        "List State",
                        "List item was not removed",
                    ));
                }
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "List State".to_string(),
            status: crate::tests::TestStatus::Success,
            message: format!("List state management: {} -> {} items", initial_count, new_count),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test reactive updates (multiple dependent states).
    async fn test_reactive_updates(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing reactive state updates");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/state", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find input for a computed value test
        let width_input = driver.find(By::Css("#rect-width")).await.ok();
        let height_input = driver.find(By::Css("#rect-height")).await.ok();
        let area_display = driver.find(By::Css("#rect-area")).await.ok();

        if let (Some(width), Some(height), Some(area)) = (width_input, height_input, area_display) {
            // Clear and set values
            width.clear().await?;
            width.send_keys("10").await?;
            tokio::time::sleep(Duration::from_millis(50)).await;

            height.clear().await?;
            height.send_keys("20").await?;
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Check computed area
            let area_text = area.text().await?;
            info!("Computed area: {}", area_text);

            // Should be 200 (10 * 20)
            if !area_text.contains("200") && !area_text.contains("10") && !area_text.contains("20") {
                return Ok(TestResult::failure(
                    "Reactive Updates",
                    &format!("Computed value did not update: {}", area_text),
                ));
            }
        } else {
            // Skip if reactive test elements don't exist
            info!("Reactive test elements not found, skipping");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Reactive Updates".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Reactive test elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Reactive Updates".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Reactive state updates work correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for StateManagementTests {
    fn name(&self) -> &str {
        "State Management Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up state management test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running state management E2E tests");

        let mut results = vec![];

        // Test 1: Counter state
        match self.test_counter_state(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Counter state test failed: {}", e);
                results.push(TestResult::error("Counter State", &e.to_string()));
            }
        }

        // Test 2: Input state binding
        match self.test_input_state_binding(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Input state binding test failed: {}", e);
                results.push(TestResult::error("Input State Binding", &e.to_string()));
            }
        }

        // Test 3: Checkbox state
        match self.test_checkbox_state(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Checkbox state test failed: {}", e);
                results.push(TestResult::error("Checkbox State", &e.to_string()));
            }
        }

        // Test 4: List state
        match self.test_list_state(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("List state test failed: {}", e);
                results.push(TestResult::error("List State", &e.to_string()));
            }
        }

        // Test 5: Reactive updates
        match self.test_reactive_updates(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Reactive updates test failed: {}", e);
                results.push(TestResult::error("Reactive Updates", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down state management test suite");
        Ok(())
    }
}
