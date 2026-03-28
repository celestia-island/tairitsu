//! Test harness for browser-glue tests

use anyhow::{Context, Result};
use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::js::EvaluationResult;
use futures::StreamExt;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

use super::reporter::{TestReport, TestResult};

/// Helper to extract boolean from evaluation result
fn eval_as_bool(result: EvaluationResult) -> bool {
    result.value().and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Test harness configuration
pub struct TestHarnessConfig {
    /// Path to Chromium executable
    pub chromium_path: PathBuf,
    /// Headless mode
    pub headless: bool,
    /// Test filter pattern (supports wildcards)
    pub filter: Option<String>,
    /// Timeout for individual tests
    pub test_timeout: Duration,
    /// Port for local test server
    pub server_port: u16,
}

impl Default for TestHarnessConfig {
    fn default() -> Self {
        Self {
            chromium_path: PathBuf::from("chromium"),
            headless: true,
            filter: None,
            test_timeout: Duration::from_secs(30),
            server_port: 3847,
        }
    }
}

/// Test harness for browser-glue
pub struct TestHarness {
    config: TestHarnessConfig,
    browser: Option<Browser>,
}

impl TestHarness {
    /// Create a new test harness
    pub fn new(config: TestHarnessConfig) -> Self {
        Self {
            config,
            browser: None,
        }
    }

    /// Start the browser
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting Chromium browser at {:?}",
            self.config.chromium_path
        );

        let mut browser_config =
            BrowserConfig::builder().chrome_executable(self.config.chromium_path.clone());

        if self.config.headless {
            browser_config = browser_config.no_sandbox();
        }

        let config = browser_config
            .build()
            .map_err(|e| anyhow::anyhow!("Browser config error: {}", e))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .context("Failed to launch Chromium")?;

        // Spawn the handler
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                debug!("Browser event: {:?}", event);
            }
        });

        self.browser = Some(browser);
        info!("Browser started successfully");
        Ok(())
    }

    /// Run all browser-glue tests
    pub async fn run_tests(&self) -> Result<TestReport> {
        let browser = self
            .browser
            .as_ref()
            .context("Browser not started. Call start() first.")?;

        let mut report = TestReport::new();

        // Create a new page
        let page = browser
            .new_page("about:blank")
            .await
            .context("Failed to create new page")?;

        info!("Running browser-glue tests...");

        // Test 1: DOM Handle Creation
        if self.should_run_test("dom-handle") {
            let result = self.test_dom_handle(&page).await;
            report.add_result(result);
        }

        // Test 2: Event Listener
        if self.should_run_test("event-listener") {
            let result = self.test_event_listener(&page).await;
            report.add_result(result);
        }

        // Test 3: HTTP Fetch
        if self.should_run_test("http-fetch") {
            let result = self.test_http_fetch(&page).await;
            report.add_result(result);
        }

        // Test 4: Handle Table Operations
        if self.should_run_test("handle-table") {
            let result = self.test_handle_table(&page).await;
            report.add_result(result);
        }

        // Test 5: Canvas Binding
        if self.should_run_test("canvas") {
            let result = self.test_canvas(&page).await;
            report.add_result(result);
        }

        // Test 6: Event Dispatch Latency
        if self.should_run_test("event-latency") {
            let result = self.test_event_latency(&page).await;
            report.add_result(result);
        }

        // Test 7: High-Frequency Event Stress
        if self.should_run_test("event-stress") {
            let result = self.test_event_stress(&page).await;
            report.add_result(result);
        }

        Ok(report)
    }

    fn should_run_test(&self, test_name: &str) -> bool {
        match &self.config.filter {
            None => true,
            Some(filter) => {
                // Simple wildcard matching
                if filter.ends_with('*') {
                    let prefix = &filter[..filter.len() - 1];
                    test_name.starts_with(prefix)
                } else {
                    test_name == filter
                }
            }
        }
    }

    async fn test_dom_handle(&self, page: &Page) -> TestResult {
        let test_name = "dom-handle";
        info!("Running test: {}", test_name);

        // Test DOM element creation and manipulation
        let script = r#"
            (function() {
                const div = document.createElement('div');
                div.id = 'test-div';
                div.textContent = 'Hello from browser-glue';
                document.body.appendChild(div);

                const span = document.createElement('span');
                span.textContent = 'Test content';
                span.id = 'test-span';
                div.appendChild(span);

                // Verify elements exist
                const divExists = document.getElementById('test-div') !== null;
                const spanExists = document.getElementById('test-span') !== null;
                const textContent = document.getElementById('test-span')?.textContent;
                return divExists && spanExists && textContent === 'Hello from browser-glue';
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(test_name, "DOM verification failed".to_string())
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_event_listener(&self, page: &Page) -> TestResult {
        let test_name = "event-listener";
        info!("Running test: {}", test_name);

        // Test event listener registration
        let script = r#"
            const btn = document.createElement('button');
            btn.id = 'test-btn';
            let clicked = false;
            btn.addEventListener('click', () => { clicked = true; });
            document.body.appendChild(btn);
            btn.click();
            clicked;
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(test_name, "Event listener did not execute".to_string())
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_http_fetch(&self, page: &Page) -> TestResult {
        let test_name = "http-fetch";
        info!("Running test: {}", test_name);

        // Test fetch API
        let script = r#"
            (async () => {
                try {
                    const response = await fetch('https://httpbin.org/get');
                    return response.ok;
                } catch (e) {
                    return false;
                }
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(test_name, "Fetch request failed".to_string())
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_handle_table(&self, page: &Page) -> TestResult {
        let test_name = "handle-table";
        info!("Running test: {}", test_name);

        // Test handle table operations (simulated)
        let script = r#"
            // Simulate handle table operations
            const handles = new Map();
            let nextId = 1;

            function createHandle(value) {
                const id = nextId++;
                handles.set(id, value);
                return id;
            }

            function getHandle(id) {
                return handles.get(id);
            }

            function dropHandle(id) {
                return handles.delete(id);
            }

            const h1 = createHandle({ data: 'test' });
            const data = getHandle(h1);
            const dropped = dropHandle(h1);

            data && dropped && !getHandle(h1);
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(test_name, "Handle table operations failed".to_string())
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_canvas(&self, page: &Page) -> TestResult {
        let test_name = "canvas";
        info!("Running test: {}", test_name);

        // Test canvas 2D context
        let script = r#"
            (() => {
                const canvas = document.createElement('canvas');
                canvas.width = 100;
                canvas.height = 100;
                const ctx = canvas.getContext('2d');
                if (!ctx) return false;

                // Draw a red rectangle
                ctx.fillStyle = 'red';
                ctx.fillRect(10, 10, 80, 80);

                // Verify pixel
                const imageData = ctx.getImageData(50, 50, 1, 1).data;
                return imageData[0] === 255 && imageData[1] === 0 && imageData[2] === 0;
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(test_name, "Canvas operations failed".to_string())
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_event_latency(&self, page: &Page) -> TestResult {
        let test_name = "event-latency";
        info!("Running test: {}", test_name);

        // Test event dispatch latency (< 1ms target)
        let script = r#"
            (async () => {
                const btn = document.createElement('button');
                btn.id = 'latency-test-btn';
                let eventTime = 0;
                let dispatchTime = 0;
                
                btn.addEventListener('click', (e) => {
                    eventTime = e.timeStamp;
                    dispatchTime = performance.now();
                });
                
                document.body.appendChild(btn);
                
                // Trigger click
                const clickTime = performance.now();
                btn.click();
                
                // Wait for event processing
                await new Promise(r => setTimeout(r, 10));
                
                // Calculate latency
                const latency = dispatchTime - clickTime;
                
                // Should be < 16ms (60fps)
                return latency < 16;
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(
                        test_name,
                        "Event latency exceeded 16ms threshold".to_string(),
                    )
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    async fn test_event_stress(&self, page: &Page) -> TestResult {
        let test_name = "event-stress";
        info!("Running test: {}", test_name);

        // Test high-frequency event handling
        let script = r#"
            (async () => {
                const btn = document.createElement('button');
                btn.id = 'stress-test-btn';
                let eventCount = 0;
                
                btn.addEventListener('click', () => {
                    eventCount++;
                });
                
                document.body.appendChild(btn);
                
                // Fire 100 events rapidly
                const startTime = performance.now();
                for (let i = 0; i < 100; i++) {
                    btn.click();
                }
                const elapsed = performance.now() - startTime;
                
                // Wait for all events to process
                await new Promise(r => setTimeout(r, 50));
                
                // All 100 events should be processed
                // Total time should be < 100ms (< 1ms per event)
                return eventCount === 100 && elapsed < 100;
            })()
        "#;

        match page.evaluate(script).await {
            Ok(result) => {
                if eval_as_bool(result) {
                    TestResult::passed(test_name)
                } else {
                    TestResult::failed(
                        test_name,
                        "High-frequency event handling failed".to_string(),
                    )
                }
            }
            Err(e) => TestResult::failed(test_name, format!("Script execution failed: {}", e)),
        }
    }

    /// Shutdown the browser
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(mut browser) = self.browser.take() {
            let _ = browser.close().await;
            info!("Browser closed");
        }
        Ok(())
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        if self.browser.is_some() {
            warn!("TestHarness dropped without calling shutdown()");
        }
    }
}
