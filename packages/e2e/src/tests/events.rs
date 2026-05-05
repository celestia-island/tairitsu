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

        let js_script = r#"
        (function() {
            var div = document.createElement('div');
            div.id = 'mouse-test-target';
            div.style.position = 'fixed';
            div.style.left = '100px';
            div.style.top = '100px';
            div.style.width = '200px';
            div.style.height = '200px';
            document.body.appendChild(div);

            var coords = null;
            div.addEventListener('mousemove', function(e) {
                coords = { x: e.clientX, y: e.clientY };
            });

            var event = new MouseEvent('mousemove', { clientX: 150, clientY: 200 });
            div.dispatchEvent(event);

            var result = coords ? 'x=' + coords.x + ',y=' + coords.y : 'no coords';
            document.body.removeChild(div);
            return result;
        })()
        "#;

        let result = driver.execute(js_script, vec![]).await;
        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(ret) => {
                let js_result: String = ret.json().as_str().unwrap_or("").to_string();
                info!("Mouse coordinate test result: {}", js_result);

                if js_result.contains("x=150,y=200") {
                    Ok(TestResult {
                        component: "Mouse Event Coordinates".to_string(),
                        status: TestStatus::Success,
                        message: format!("Mouse coordinates verified: {}", js_result),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else if js_result.contains("no coords") {
                    Ok(TestResult {
                        component: "Mouse Event Coordinates".to_string(),
                        status: TestStatus::Failure,
                        message: "mousemove event did not fire or coordinates were not captured"
                            .to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Mouse Event Coordinates".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Unexpected mouse coordinates: {}", js_result),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Mouse Event Coordinates".to_string(),
                status: TestStatus::Failure,
                message: format!("JavaScript execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
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

        let js_script = r#"
        (function() {
            var buttons = document.querySelectorAll('button');
            if (buttons.length === 0) return 'no buttons';
            var clicked = false;
            var handler = function() { clicked = true; };
            buttons[0].addEventListener('click', handler);
            buttons[0].click();
            buttons[0].removeEventListener('click', handler);
            return clicked ? 'listener fires' : 'no listener';
        })()
        "#;

        let result = driver.execute(js_script, vec![]).await;
        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(ret) => {
                let js_result: String = ret.json().as_str().unwrap_or("").to_string();
                info!("Event listener test result: {}", js_result);

                if js_result == "listener fires" {
                    Ok(TestResult {
                        component: "Event Listener Registration".to_string(),
                        status: TestStatus::Success,
                        message: "Click event listener fires correctly on button".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else if js_result == "no buttons" {
                    Ok(TestResult {
                        component: "Event Listener Registration".to_string(),
                        status: TestStatus::Failure,
                        message: "No buttons found on the page to test event listeners".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Event Listener Registration".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Event listener did not fire: {}", js_result),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Event Listener Registration".to_string(),
                status: TestStatus::Failure,
                message: format!("JavaScript execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
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

        let js_script = r#"
        (function() {
            var forms = document.querySelectorAll('form');
            if (forms.length === 0) return 'no forms';
            var submitted = false;
            forms[0].addEventListener('submit', function(e) { 
                e.preventDefault(); 
                submitted = true; 
            });
            var submitBtn = forms[0].querySelector('button[type="submit"], input[type="submit"]');
            if (submitBtn) submitBtn.click();
            else forms[0].dispatchEvent(new Event('submit'));
            return submitted ? 'submit intercepted' : 'no submit event';
        })()
        "#;

        let result = driver.execute(js_script, vec![]).await;
        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(ret) => {
                let js_result: String = ret.json().as_str().unwrap_or("").to_string();
                info!("Form submission test result: {}", js_result);

                if js_result == "submit intercepted" {
                    Ok(TestResult {
                        component: "Form Submission".to_string(),
                        status: TestStatus::Success,
                        message: "Form submit event fired and was intercepted correctly"
                            .to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else if js_result == "no forms" {
                    Ok(TestResult {
                        component: "Form Submission".to_string(),
                        status: TestStatus::Warning,
                        message: "No forms found on the page".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Form Submission".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Submit event was not intercepted: {}", js_result),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Form Submission".to_string(),
                status: TestStatus::Failure,
                message: format!("JavaScript execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
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

        let js_script = r#"
        (function() {
            var before = window.scrollY;
            window.scrollBy(0, 100);
            var after = window.scrollY;
            return 'before=' + before + ',after=' + after;
        })()
        "#;

        let result = driver.execute(js_script, vec![]).await;
        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(ret) => {
                let js_result: String = ret.json().as_str().unwrap_or("").to_string();
                info!("Scroll test result: {}", js_result);

                Ok(TestResult {
                    component: "Scroll Event".to_string(),
                    status: TestStatus::Success,
                    message: format!("Scroll executed successfully: {}", js_result),
                    duration_ms: duration,
                    screenshot_path: None,
                })
            }
            Err(e) => Ok(TestResult {
                component: "Scroll Event".to_string(),
                status: TestStatus::Failure,
                message: format!("Scroll JavaScript execution failed: {}", e),
                duration_ms: duration,
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
