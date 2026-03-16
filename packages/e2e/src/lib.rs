pub mod tests;

pub use tests::{Test, TestResult, TestStatus};

use anyhow::Result;
use thirtyfour::WebDriver;
use tracing::info;

pub async fn run_all_tests(driver: &WebDriver) -> Result<Vec<TestResult>> {
    info!("Running all Tairitsu E2E tests...\n");

    let mut results = vec![];

    match tests::Test::run_with_driver(&tests::BasicComponentsTests, driver).await {
        Ok(result) => results.push(result),
        Err(e) => {
            eprintln!("Basic components test suite failed: {}", e);
            results.push(TestResult::error("BasicComponents", e.to_string().as_str()));
        }
    }

    info!("\n=== E2E Test Results ===");
    for result in &results {
        info!("{}: {}", result.component, result.message);
        match &result.status {
            TestStatus::Success => info!("  Status: ✅ PASSED"),
            TestStatus::Failure => info!("  Status: ❌ FAILED"),
            TestStatus::Error(msg) => {
                info!("  Status: ⚠️  ERROR - {}", msg)
            }
        }
    }
    info!("=== End of Test Results ===\n");

    Ok(results)
}
