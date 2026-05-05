//! Test result reporting

use std::{fmt, time::Duration};

/// Result of a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Test name
    pub name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration of the test
    pub duration: Duration,
}

impl TestResult {
    /// Create a passed test result
    pub fn passed(name: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            error: None,
            duration: Duration::ZERO,
        }
    }

    /// Create a failed test result
    pub fn failed(name: &str, error: String) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            error: Some(error),
            duration: Duration::ZERO,
        }
    }

    /// Create a skipped test result
    pub fn skipped(name: &str, reason: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            error: Some(format!("SKIPPED: {}", reason)),
            duration: Duration::ZERO,
        }
    }

    /// Set the duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = if self.passed { "✓ PASS" } else { "✗ FAIL" };
        let duration = format!("{:?}", self.duration);
        write!(f, "{} {} [{}]", status, self.name, duration)?;
        if let Some(error) = &self.error {
            write!(f, "\n    Error: {}", error)?;
        }
        Ok(())
    }
}

/// Test report containing all results
#[derive(Debug, Clone, Default)]
pub struct TestReport {
    /// All test results
    pub results: Vec<TestResult>,
    /// Total duration
    pub total_duration: Duration,
}

impl TestReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            total_duration: Duration::ZERO,
        }
    }

    /// Add a test result
    pub fn add_result(&mut self, result: TestResult) {
        self.total_duration += result.duration;
        self.results.push(result);
    }

    /// Get the number of passed tests
    pub fn passed(&self) -> usize {
        self.results.iter().filter(|r| r.passed).count()
    }

    /// Get the number of failed tests
    pub fn failed(&self) -> usize {
        self.results.iter().filter(|r| !r.passed).count()
    }

    /// Get the total number of tests
    pub fn total(&self) -> usize {
        self.results.len()
    }

    /// Check if all tests passed
    pub fn is_success(&self) -> bool {
        self.failed() == 0
    }
}

impl fmt::Display for TestReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\n{}", "=".repeat(60))?;
        writeln!(f, "Test Report")?;
        writeln!(f, "{}", "=".repeat(60))?;

        for result in &self.results {
            writeln!(f, "  {}", result)?;
        }

        writeln!(f, "{}", "-".repeat(60))?;
        writeln!(
            f,
            "Total: {} | Passed: {} | Failed: {}",
            self.total(),
            self.passed(),
            self.failed()
        )?;
        writeln!(f, "Duration: {:?}", self.total_duration)?;
        writeln!(f, "{}", "=".repeat(60))?;

        Ok(())
    }
}
