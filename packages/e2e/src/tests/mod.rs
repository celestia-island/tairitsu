pub mod async_operations;
pub mod basic_components;
pub mod build;
pub mod doctor;
pub mod error_handling;
pub mod events;
pub mod form_validation;
pub mod lifecycle;
pub mod navigation;
pub mod ssr;
pub mod state_management;
pub mod style_integration;
pub mod svg_safety;

use anyhow::Result;

pub use async_operations::AsyncOperationsTests;
pub use basic_components::BasicComponentsTests;
pub use build::BuildTests;
pub use doctor::DoctorTests;
pub use error_handling::ErrorHandlingTests;
pub use events::EventTests;
pub use form_validation::FormValidationTests;
pub use lifecycle::LifecycleTests;
pub use navigation::NavigationTests;
pub use ssr::SsrTests;
pub use state_management::StateManagementTests;
pub use style_integration::StyleIntegrationTests;
pub use svg_safety::SvgSafetyTests;
use thirtyfour::WebDriver;

pub trait Test: Send + Sync {
    fn name(&self) -> &str;

    fn setup(&self) -> Result<()> {
        Ok(())
    }

    fn run_with_driver(
        &self,
        driver: &WebDriver,
    ) -> impl std::future::Future<Output = Result<TestResult>> + Send;

    fn teardown(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Success,
    Warning,
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
        let warnings = results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Warning))
            .count();
        let errors = results
            .iter()
            .filter(|r| matches!(r.status, TestStatus::Failure | TestStatus::Error(_)))
            .count();

        let status = if errors == 0 && warnings == 0 {
            TestStatus::Success
        } else if errors == 0 {
            TestStatus::Warning
        } else if errors < total {
            TestStatus::Failure
        } else {
            TestStatus::Error(format!("{} of {} tests failed", errors, total))
        };

        Self {
            component: "Test Suite".to_string(),
            status,
            message: format!(
                "{} passed, {} warnings, {} errors",
                passed, warnings, errors
            ),
            duration_ms: 0,
            screenshot_path: None,
        }
    }
}
