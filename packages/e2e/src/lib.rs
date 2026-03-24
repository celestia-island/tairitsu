pub mod tests;

use anyhow::Result;

pub use tests::{Test, TestResult, TestStatus};
use thirtyfour::WebDriver;
use tracing::info;

pub async fn run_all_tests(driver: &WebDriver) -> Result<Vec<TestResult>> {
    info!("Running all Tairitsu E2E tests...\n");

    let mut results = vec![];

    // Basic Components Tests
    info!("Running Basic Components Tests...");
    match tests::Test::run_with_driver(&tests::BasicComponentsTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Basic components test suite failed: {}", e);
            results.push(TestResult::error("BasicComponents", e.to_string().as_str()));
        }
    }

    // Component Lifecycle Tests
    info!("Running Component Lifecycle Tests...");
    match tests::Test::run_with_driver(&tests::LifecycleTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Lifecycle test suite failed: {}", e);
            results.push(TestResult::error("LifecycleTests", e.to_string().as_str()));
        }
    }

    // Event Handling Tests
    info!("Running Event Handling Tests...");
    match tests::Test::run_with_driver(&tests::EventTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Event handling test suite failed: {}", e);
            results.push(TestResult::error("EventTests", e.to_string().as_str()));
        }
    }

    // Build Process Tests (don't require WebDriver)
    info!("Running Build Process Tests...");
    match tests::Test::run_with_driver(&tests::BuildTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Build process test suite failed: {}", e);
            results.push(TestResult::error("BuildTests", e.to_string().as_str()));
        }
    }

    // Doctor Command Tests (don't require WebDriver)
    info!("Running Doctor Command Tests...");
    match tests::Test::run_with_driver(&tests::DoctorTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Doctor command test suite failed: {}", e);
            let error_msg: String = e.to_string();
            results.push(TestResult::error("DoctorTests", error_msg.as_str()));
        }
    }

    // Error Handling Tests (don't require WebDriver)
    info!("Running Error Handling Tests...");
    match tests::Test::run_with_driver(&tests::ErrorHandlingTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Error handling test suite failed: {}", e);
            let error_msg: String = e.to_string();
            results.push(TestResult::error("ErrorHandlingTests", error_msg.as_str()));
        }
    }

    // SVG Safety Tests (don't require WebDriver)
    info!("Running SVG Safety Tests...");
    match tests::Test::run_with_driver(&tests::SvgSafetyTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("SVG safety test suite failed: {}", e);
            let error_msg: String = e.to_string();
            results.push(TestResult::error("SvgSafetyTests", error_msg.as_str()));
        }
    }

    // SSR Tests (don't require WebDriver)
    info!("Running SSR Tests...");
    match tests::Test::run_with_driver(&tests::SsrTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("SSR test suite failed: {}", e);
            let error_msg: String = e.to_string();
            results.push(TestResult::error("SsrTests", error_msg.as_str()));
        }
    }

    info!("\n=== E2E Test Results ===");
    for result in &results {
        info!("{}: {}", result.component, result.message);
        match &result.status {
            TestStatus::Success => info!("  Status: PASSED"),
            TestStatus::Warning => info!("  Status: WARNING"),
            TestStatus::Failure => info!("  Status: FAILED"),
            TestStatus::Error(msg) => {
                info!("  Status: ERROR - {}", msg)
            }
        }
    }
    info!("=== End of Test Results ===\n");

    Ok(results)
}
