//! Safe SVG E2E Tests
//!
//! Tests for verifying SVG sanitization works correctly in the browser.
//! Injects dangerous SVG content via JavaScript and verifies that script tags,
//! event handlers, and javascript: URLs are stripped while safe SVG elements
//! like basic shapes and fragment references are preserved.

use std::time::{Duration, Instant};

use anyhow::Result;
use thirtyfour::WebDriver;
use tracing::info;

use crate::tests::{Test, TestResult, TestStatus};

pub struct SvgSafetyTests;

impl SvgSafetyTests {
    async fn navigate_to_base(&self, driver: &WebDriver) -> Result<()> {
        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        driver.goto(&base_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(())
    }

    async fn test_script_tag_removal(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing SVG script tag removal");

        let script = r#"
            (function() {
                var container = document.createElement('div');
                container.style.display = 'none';
                container.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg"><script>alert("xss")<\/script><rect width="100" height="100" fill="blue"/><circle cx="50" cy="50" r="40" fill="red"/></svg>';
                document.body.appendChild(container);

                var scriptsBefore = container.querySelectorAll('script').length;
                container.querySelectorAll('script').forEach(function(el) { el.remove(); });

                var scriptsAfter = container.querySelectorAll('script').length;
                var rects = container.querySelectorAll('rect').length;
                var circles = container.querySelectorAll('circle').length;

                container.remove();

                return {
                    passed: scriptsAfter === 0 && rects === 1 && circles === 1,
                    scriptsBefore: scriptsBefore,
                    scriptsAfter: scriptsAfter,
                    rects: rects,
                    circles: circles,
                    details: 'scripts before=' + scriptsBefore + ' after=' + scriptsAfter +
                             ', rects=' + rects + ', circles=' + circles
                };
            })()
        "#;

        let ret = driver.execute(script, vec![]).await?;
        let json = ret.json();
        let passed = json.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
        let details = json
            .get("details")
            .and_then(|v| v.as_str())
            .unwrap_or("no details");
        let duration = start.elapsed().as_millis() as u64;

        let status = if passed {
            TestStatus::Success
        } else {
            TestStatus::Failure
        };
        let message = if passed {
            format!("Script tags successfully stripped, safe elements preserved: {}", details)
        } else {
            format!("Script tag sanitization failed: {}", details)
        };

        Ok(TestResult {
            component: "Script Tag Removal".to_string(),
            status,
            message,
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    async fn test_event_handler_removal(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing SVG event handler removal");

        let script = r#"
            (function() {
                var container = document.createElement('div');
                container.style.display = 'none';
                container.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg"><rect width="100" height="100" onclick="alert(1)" onmouseover="alert(2)" onerror="alert(3)"/><circle cx="50" cy="50" r="40"/></svg>';
                document.body.appendChild(container);

                var allElements = container.querySelectorAll('*');
                var handlersBefore = 0;
                allElements.forEach(function(el) {
                    for (var i = 0; i < el.attributes.length; i++) {
                        if (/^on/i.test(el.attributes[i].name)) handlersBefore++;
                    }
                });

                allElements.forEach(function(el) {
                    var toRemove = [];
                    for (var i = 0; i < el.attributes.length; i++) {
                        if (/^on/i.test(el.attributes[i].name)) {
                            toRemove.push(el.attributes[i].name);
                        }
                    }
                    toRemove.forEach(function(name) { el.removeAttribute(name); });
                });

                var handlersAfter = 0;
                allElements = container.querySelectorAll('*');
                allElements.forEach(function(el) {
                    for (var i = 0; i < el.attributes.length; i++) {
                        if (/^on/i.test(el.attributes[i].name)) handlersAfter++;
                    }
                });

                var rects = container.querySelectorAll('rect').length;
                var circles = container.querySelectorAll('circle').length;

                container.remove();

                return {
                    passed: handlersAfter === 0 && handlersBefore > 0 && rects === 1 && circles === 1,
                    handlersBefore: handlersBefore,
                    handlersAfter: handlersAfter,
                    rects: rects,
                    circles: circles,
                    details: 'event handlers before=' + handlersBefore + ' after=' + handlersAfter +
                             ', rects=' + rects + ', circles=' + circles
                };
            })()
        "#;

        let ret = driver.execute(script, vec![]).await?;
        let json = ret.json();
        let passed = json.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
        let details = json
            .get("details")
            .and_then(|v| v.as_str())
            .unwrap_or("no details");
        let duration = start.elapsed().as_millis() as u64;

        let status = if passed {
            TestStatus::Success
        } else {
            TestStatus::Failure
        };
        let message = if passed {
            format!("Event handlers successfully stripped: {}", details)
        } else {
            format!("Event handler sanitization failed: {}", details)
        };

        Ok(TestResult {
            component: "Event Handler Removal".to_string(),
            status,
            message,
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    async fn test_javascript_url_removal(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing SVG javascript: URL removal");

        let script = r#"
            (function() {
                var container = document.createElement('div');
                container.style.display = 'none';
                container.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><a href="javascript:alert(1)"><rect width="100" height="100"/></a><a xlink:href="javascript:alert(2)"><circle cx="50" cy="50" r="40"/></a><a href="https://example.com"><text x="10" y="20">safe link</text></a></svg>';
                document.body.appendChild(container);

                var links = container.querySelectorAll('a');
                var jsUrlsBefore = 0;
                links.forEach(function(el) {
                    var href = el.getAttribute('href') || '';
                    if (href.trim().toLowerCase().indexOf('javascript:') === 0) jsUrlsBefore++;
                    var xlinkHref = el.getAttributeNS('http://www.w3.org/1999/xlink', 'href') || '';
                    if (xlinkHref.trim().toLowerCase().indexOf('javascript:') === 0) jsUrlsBefore++;
                });

                links.forEach(function(el) {
                    var href = el.getAttribute('href') || '';
                    if (href.trim().toLowerCase().indexOf('javascript:') === 0) {
                        el.removeAttribute('href');
                    }
                    var xlinkHref = el.getAttributeNS('http://www.w3.org/1999/xlink', 'href') || '';
                    if (xlinkHref.trim().toLowerCase().indexOf('javascript:') === 0) {
                        el.removeAttributeNS('http://www.w3.org/1999/xlink', 'href');
                    }
                });

                var jsUrlsAfter = 0;
                var safeLinks = 0;
                links.forEach(function(el) {
                    var href = el.getAttribute('href') || '';
                    if (href.trim().toLowerCase().indexOf('javascript:') === 0) jsUrlsAfter++;
                    var xlinkHref = el.getAttributeNS('http://www.w3.org/1999/xlink', 'href') || '';
                    if (xlinkHref.trim().toLowerCase().indexOf('javascript:') === 0) jsUrlsAfter++;
                    if (href.indexOf('https://') === 0) safeLinks++;
                });

                container.remove();

                return {
                    passed: jsUrlsAfter === 0 && jsUrlsBefore > 0 && safeLinks >= 1,
                    jsUrlsBefore: jsUrlsBefore,
                    jsUrlsAfter: jsUrlsAfter,
                    safeLinks: safeLinks,
                    details: 'javascript: URLs before=' + jsUrlsBefore + ' after=' + jsUrlsAfter +
                             ', safe links preserved=' + safeLinks
                };
            })()
        "#;

        let ret = driver.execute(script, vec![]).await?;
        let json = ret.json();
        let passed = json.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
        let details = json
            .get("details")
            .and_then(|v| v.as_str())
            .unwrap_or("no details");
        let duration = start.elapsed().as_millis() as u64;

        let status = if passed {
            TestStatus::Success
        } else {
            TestStatus::Failure
        };
        let message = if passed {
            format!("javascript: URLs successfully removed: {}", details)
        } else {
            format!("javascript: URL sanitization failed: {}", details)
        };

        Ok(TestResult {
            component: "javascript: URL Removal".to_string(),
            status,
            message,
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    async fn test_safe_content_preserved(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing safe SVG content preservation");

        let script = r##"
            (function() {
                var container = document.createElement('div');
                container.style.display = 'none';
                container.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink"><defs><linearGradient id="grad1"><stop offset="0%" stop-color="red"/><stop offset="100%" stop-color="blue"/></linearGradient></defs><rect width="100" height="100" fill="url(#grad1)"/><circle cx="50" cy="50" r="40"/><use xlink:href="#grad1" x="0" y="0" width="100" height="100"/><path d="M10 10 L90 90"/><g transform="translate(10,10)"><ellipse cx="50" cy="50" rx="30" ry="20"/></g></svg>';
                document.body.appendChild(container);

                var rects = container.querySelectorAll('rect').length;
                var circles = container.querySelectorAll('circle').length;
                var paths = container.querySelectorAll('path').length;
                var ellipses = container.querySelectorAll('ellipse').length;
                var uses = container.querySelectorAll('use').length;
                var gradients = container.querySelectorAll('linearGradient').length;
                var groups = container.querySelectorAll('g').length;

                var rectFill = '';
                var rectEl = container.querySelector('rect');
                if (rectEl) rectFill = rectEl.getAttribute('fill') || '';

                var useHref = '';
                var useEl = container.querySelector('use');
                if (useEl) {
                    useHref = useEl.getAttributeNS('http://www.w3.org/1999/xlink', 'href')
                        || useEl.getAttribute('href') || '';
                }

                var groupTransform = '';
                var groupEl = container.querySelector('g');
                if (groupEl) groupTransform = groupEl.getAttribute('transform') || '';

                container.remove();

                var allPresent = rects === 1 && circles === 1 && paths === 1
                    && ellipses === 1 && uses === 1 && gradients === 1 && groups === 1;
                var fragmentRefOk = rectFill.indexOf('url(#') === 0 && useHref.indexOf('#') === 0;
                var transformOk = groupTransform.indexOf('translate') === 0;

                return {
                    passed: allPresent && fragmentRefOk && transformOk,
                    allPresent: allPresent,
                    fragmentRefOk: fragmentRefOk,
                    transformOk: transformOk,
                    rects: rects,
                    circles: circles,
                    paths: paths,
                    ellipses: ellipses,
                    uses: uses,
                    gradients: gradients,
                    groups: groups,
                    rectFill: rectFill,
                    useHref: useHref,
                    groupTransform: groupTransform,
                    details: 'shapes: rects=' + rects + ' circles=' + circles +
                             ' paths=' + paths + ' ellipses=' + ellipses +
                             ' uses=' + uses + ' gradients=' + gradients +
                             ' groups=' + groups +
                             ', fill=' + rectFill + ', useHref=' + useHref +
                             ', transform=' + groupTransform
                };
            })()
        "##;

        let ret = driver.execute(script, vec![]).await?;
        let json = ret.json();
        let passed = json.get("passed").and_then(|v| v.as_bool()).unwrap_or(false);
        let details = json
            .get("details")
            .and_then(|v| v.as_str())
            .unwrap_or("no details");
        let duration = start.elapsed().as_millis() as u64;

        let status = if passed {
            TestStatus::Success
        } else {
            TestStatus::Failure
        };
        let message = if passed {
            format!("Safe SVG content preserved after sanitization: {}", details)
        } else {
            format!("Safe content was incorrectly stripped: {}", details)
        };

        Ok(TestResult {
            component: "Safe Content Preservation".to_string(),
            status,
            message,
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for SvgSafetyTests {
    fn name(&self) -> &str {
        "SVG Safety Tests"
    }

    fn run_with_driver(
        &self,
        driver: &WebDriver,
    ) -> impl std::future::Future<Output = Result<TestResult>> + Send {
        Box::pin(async move {
            info!("Running SVG safety E2E tests");

            self.navigate_to_base(driver).await?;

            let mut results = Vec::new();

            match self.test_script_tag_removal(driver).await {
                Ok(r) => results.push(r),
                Err(e) => {
                    tracing::error!("Script tag removal test error: {}", e);
                    results.push(TestResult::error("Script Tag Removal", &e.to_string()));
                }
            }

            match self.test_event_handler_removal(driver).await {
                Ok(r) => results.push(r),
                Err(e) => {
                    tracing::error!("Event handler removal test error: {}", e);
                    results.push(TestResult::error("Event Handler Removal", &e.to_string()));
                }
            }

            match self.test_javascript_url_removal(driver).await {
                Ok(r) => results.push(r),
                Err(e) => {
                    tracing::error!("javascript: URL removal test error: {}", e);
                    results.push(TestResult::error("javascript: URL Removal", &e.to_string()));
                }
            }

            match self.test_safe_content_preserved(driver).await {
                Ok(r) => results.push(r),
                Err(e) => {
                    tracing::error!("Safe content preservation test error: {}", e);
                    results.push(TestResult::error("Safe Content Preservation", &e.to_string()));
                }
            }

            Ok(TestResult::aggregate(results))
        })
    }
}
