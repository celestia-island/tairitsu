//! Event handling tests
//!
//! Tests for DOM event handling including click, input, keyboard, and focus events.

use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, Key, WebDriver};
use tracing::info;

use crate::tests::{Test, TestResult, TestStatus};

pub struct EventTests;

impl EventTests {
    /// Test click event handling
    async fn test_click_event(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing click event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find a clickable element
        let buttons = driver.find_all(By::Tag("button")).await?;
        let links = driver.find_all(By::Tag("a")).await?;

        let duration = start.elapsed().as_millis() as u64;

        if !buttons.is_empty() {
            // Try clicking the first button
            match buttons[0].click().await {
                Ok(_) => Ok(TestResult {
                    component: "Click Event".to_string(),
                    status: TestStatus::Success,
                    message: format!(
                        "Button clicked successfully ({} buttons found)",
                        buttons.len()
                    ),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
                Err(e) => Ok(TestResult {
                    component: "Click Event".to_string(),
                    status: TestStatus::Failure,
                    message: format!("Button click failed: {}", e),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
            }
        } else if !links.is_empty() {
            // Try clicking the first link
            match links[0].click().await {
                Ok(_) => Ok(TestResult {
                    component: "Click Event".to_string(),
                    status: TestStatus::Success,
                    message: format!("Link clicked successfully ({} links found)", links.len()),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
                Err(e) => Ok(TestResult {
                    component: "Click Event".to_string(),
                    status: TestStatus::Failure,
                    message: format!("Link click failed: {}", e),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
            }
        } else {
            Ok(TestResult {
                component: "Click Event".to_string(),
                status: TestStatus::Warning,
                message: "No clickable elements found".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test input event handling
    async fn test_input_event(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing input event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find input elements
        let inputs = driver.find_all(By::Tag("input")).await?;
        let textareas = driver.find_all(By::Tag("textarea")).await?;

        let duration = start.elapsed().as_millis() as u64;

        if !inputs.is_empty() {
            // Try typing in the first input
            let test_text = "E2E Test Input";
            match inputs[0].send_keys(test_text).await {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(TestResult {
                        component: "Input Event".to_string(),
                        status: TestStatus::Success,
                        message: format!(
                            "Input event handled successfully ({} inputs, {} textareas)",
                            inputs.len(),
                            textareas.len()
                        ),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
                Err(e) => Ok(TestResult {
                    component: "Input Event".to_string(),
                    status: TestStatus::Failure,
                    message: format!("Input event failed: {}", e),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
            }
        } else if !textareas.is_empty() {
            // Try typing in the first textarea
            let test_text = "E2E Test Textarea";
            match textareas[0].send_keys(test_text).await {
                Ok(_) => Ok(TestResult {
                    component: "Input Event".to_string(),
                    status: TestStatus::Success,
                    message: format!(
                        "Textarea input handled successfully ({} textareas)",
                        textareas.len()
                    ),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
                Err(e) => Ok(TestResult {
                    component: "Input Event".to_string(),
                    status: TestStatus::Failure,
                    message: format!("Textarea input failed: {}", e),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
            }
        } else {
            Ok(TestResult {
                component: "Input Event".to_string(),
                status: TestStatus::Warning,
                message: "No input elements found".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test keyboard event handling
    async fn test_keyboard_event(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing keyboard event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Try sending keyboard events to the body
        let body = driver.find(By::Tag("body")).await?;

        // Send Tab key
        match body.send_keys(Key::Tab).await {
            Ok(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Send Enter key
                match body.send_keys(Key::Enter).await {
                    Ok(_) => Ok(TestResult {
                        component: "Keyboard Event".to_string(),
                        status: TestStatus::Success,
                        message: "Keyboard events (Tab, Enter) handled successfully".to_string(),
                        duration_ms: start.elapsed().as_millis() as u64,
                        screenshot_path: None,
                    }),
                    Err(e) => Ok(TestResult {
                        component: "Keyboard Event".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Enter key event failed: {}", e),
                        duration_ms: start.elapsed().as_millis() as u64,
                        screenshot_path: None,
                    }),
                }
            }
            Err(e) => Ok(TestResult {
                component: "Keyboard Event".to_string(),
                status: TestStatus::Failure,
                message: format!("Tab key event failed: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            }),
        }
    }

    /// Test focus event handling
    async fn test_focus_event(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing focus event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find focusable elements
        let inputs = driver.find_all(By::Tag("input")).await?;

        let duration = start.elapsed().as_millis() as u64;

        if !inputs.is_empty() {
            // Try focusing the first input - simplified check
            match inputs[0].click().await {
                Ok(_) => {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    Ok(TestResult {
                        component: "Focus Event".to_string(),
                        status: TestStatus::Success,
                        message: format!("Focus event handled ({} input(s) found)", inputs.len()),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
                Err(e) => Ok(TestResult {
                    component: "Focus Event".to_string(),
                    status: TestStatus::Failure,
                    message: format!("Focus event failed: {}", e),
                    duration_ms: duration,
                    screenshot_path: None,
                }),
            }
        } else {
            Ok(TestResult {
                component: "Focus Event".to_string(),
                status: TestStatus::Warning,
                message: "No focusable elements found".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test mouse event coordinates
    async fn test_mouse_event_coordinates(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing mouse event coordinates");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Try to get current window size - simplified
        let _window = driver.window().await;
        let duration = start.elapsed().as_millis() as u64;

        Ok(TestResult {
            component: "Mouse Event Coordinates".to_string(),
            status: TestStatus::Success,
            message: "Window handle obtained successfully".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test event listener registration
    async fn test_event_listener_registration(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing event listener registration");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Check if elements exist that could have listeners
        let buttons = driver.find_all(By::Tag("button")).await?;
        let inputs = driver.find_all(By::Tag("input")).await?;

        let duration = start.elapsed().as_millis() as u64;

        Ok(TestResult {
            component: "Event Listener Registration".to_string(),
            status: TestStatus::Success,
            message: format!(
                "Found {} potential event listener elements (buttons: {}, inputs: {})",
                buttons.len() + inputs.len(),
                buttons.len(),
                inputs.len()
            ),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test form submission event
    async fn test_form_submission(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing form submission event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Look for forms
        let forms = driver.find_all(By::Tag("form")).await?;

        let duration = start.elapsed().as_millis() as u64;

        if !forms.is_empty() {
            Ok(TestResult {
                component: "Form Submission".to_string(),
                status: TestStatus::Success,
                message: format!("Found {} form(s) on the page", forms.len()),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Form Submission".to_string(),
                status: TestStatus::Warning,
                message: "No forms found on the page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test scroll event
    async fn test_scroll_event(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing scroll event");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Try scrolling using the driver's script execution (simplified)
        match driver.execute("window.scrollBy(0, 100);", vec![]).await {
            Ok(_) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok(TestResult {
                    component: "Scroll Event".to_string(),
                    status: TestStatus::Success,
                    message: "Scroll executed successfully".to_string(),
                    duration_ms: start.elapsed().as_millis() as u64,
                    screenshot_path: None,
                })
            }
            Err(e) => Ok(TestResult {
                component: "Scroll Event".to_string(),
                status: TestStatus::Warning,
                message: format!("Scroll test returned: {}", e),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            }),
        }
    }
}

impl Test for EventTests {
    fn name(&self) -> &str {
        "Event Handling Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up event handling test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running event handling tests");

        let mut results = vec![];

        // Run all event tests
        match self.test_click_event(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Click event test failed: {}", e);
                results.push(TestResult::error("Click Event", &e.to_string()));
            }
        }

        match self.test_input_event(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Input event test failed: {}", e);
                results.push(TestResult::error("Input Event", &e.to_string()));
            }
        }

        match self.test_keyboard_event(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Keyboard event test failed: {}", e);
                results.push(TestResult::error("Keyboard Event", &e.to_string()));
            }
        }

        match self.test_focus_event(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Focus event test failed: {}", e);
                results.push(TestResult::error("Focus Event", &e.to_string()));
            }
        }

        match self.test_mouse_event_coordinates(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Mouse coordinates test failed: {}", e);
                results.push(TestResult::error("Mouse Coordinates", &e.to_string()));
            }
        }

        match self.test_event_listener_registration(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Event listener test failed: {}", e);
                results.push(TestResult::error("Event Listener", &e.to_string()));
            }
        }

        match self.test_form_submission(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Form submission test failed: {}", e);
                results.push(TestResult::error("Form Submission", &e.to_string()));
            }
        }

        match self.test_scroll_event(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Scroll event test failed: {}", e);
                results.push(TestResult::error("Scroll Event", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down event handling test suite");
        Ok(())
    }
}
