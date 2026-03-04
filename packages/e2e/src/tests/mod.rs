pub mod basic_components;

pub use basic_components::BasicComponentsTests;

use thirtyfour::WebDriver;
use anyhow::Result;
use tracing::info;

pub trait Test {
    fn name(&self) -> &str;
    
    fn setup(&self) -> Result<()> {
        Ok(())
    }
    
    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult>;
    
    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum TestStatus {
    Success,
    Failure,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub component: String,
    pub status: TestStatus,
    pub message: String,
    pub duration_ms: u64,
    pub screenshot_path: Option<String>,
}

impl TestResult {
    pub fn success(component: &str, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status: TestStatus::Success,
            message: message.to_string(),
            duration_ms: 0,
            screenshot_path: None,
        }
    }
    
    pub fn failure(component: &str, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status: TestStatus::Failure,
            message: message.to_string(),
            duration_ms: 1,
            screenshot_path: None,
        }
    }
    
    pub fn error(component: &str, error_msg: &str) -> Self {
        Self {
            component: component.to_string(),
            status: TestStatus::Error(error_msg.to_string()),
            message: error_msg.to_string(),
            duration_ms: 1,
            screenshot_path: None,
        }
    }
    
    pub fn aggregate(results: Vec<TestResult>) -> Self {
        let total = results.len();
        let passed = results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Success))
            .count();
        let failed = total - passed;
        
        let status = if failed == 0 {
            TestStatus::Success
        } else if failed < total {
            TestStatus::Failure
        } else {
            TestStatus::Error(format!("{} of {} tests failed", failed, total))
        };
        
        Self {
            component: "Test Suite".to_string(),
            status,
            message: format!("{} passed, {} failed", passed, failed),
            duration_ms: 0,
            screenshot_path: None,
        }
    }
}
