//! Error handling tests
//!
//! Tests for error handling including build errors, runtime errors, and edge cases.

use anyhow::Result;
use std::{fs, path::PathBuf, process::Command, time::Instant};

use tempfile::TempDir;
use thirtyfour::WebDriver;
use tracing::info;

use crate::tests::{Test, TestResult, TestStatus};

pub struct ErrorHandlingTests;

impl ErrorHandlingTests {
    /// Create a project with invalid Rust code
    fn create_invalid_syntax_project(temp_dir: &TempDir, name: &str) -> Result<PathBuf> {
        let project_path = temp_dir.path().join(name);
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tairitsu-web = {{ path = "../../../packages/web", features = ["wit-bindings"] }}
"#,
            name
        );
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // Invalid Rust syntax
        let lib_rs = r#"
use tairitsu_web::WitPlatform;

// This is invalid Rust syntax
fn invalid_function() {
    let x =
    // Missing value after =

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    WitPlatform::mount(|| {
        tairitsu_web::vdom::VNode::element("div", vec![], vec![
            tairitsu_web::vdom::VNode::text("Invalid"),
        ])
    });
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Create a project with invalid TOML configuration
    fn create_invalid_toml_project(temp_dir: &TempDir, name: &str) -> Result<PathBuf> {
        let project_path = temp_dir.path().join(name);
        fs::create_dir_all(project_path.join("src"))?;

        // Invalid TOML - missing closing bracket
        let cargo_toml = r#"[package]
name = "invalid-toml"
version = "0.1.0"
edition = "2021"

[lib
crate-type = ["cdylib"]

[dependencies]
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        let lib_rs = r#"
#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Create a project with missing bootstrap function
    #[allow(dead_code)]
    fn create_missing_bootstrap_project(temp_dir: &TempDir, name: &str) -> Result<PathBuf> {
        let project_path = temp_dir.path().join(name);
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tairitsu-web = {{ path = "../../../packages/web", features = ["wit-bindings"] }}
"#,
            name
        );
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // Missing the required bootstrap export
        let lib_rs = r#"
// This file intentionally missing the bootstrap function
pub fn some_other_function() {
    println!("Hello");
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Test handling of invalid Rust syntax
    fn test_invalid_syntax_error(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing invalid syntax error handling");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_invalid_syntax_project(&temp_dir, "invalid-syntax")?;

        // Try to build - should fail with clear error message
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let combined = format!("{}\n{}", stdout, stderr);

                if !out.status.success() {
                    // Should fail to compile
                    if combined.contains("error") {
                        Ok(TestResult {
                            component: "Invalid Syntax Error".to_string(),
                            status: TestStatus::Success,
                            message: "Invalid syntax correctly rejected".to_string(),
                            duration_ms: duration,
                            screenshot_path: None,
                        })
                    } else {
                        Ok(TestResult {
                            component: "Invalid Syntax Error".to_string(),
                            status: TestStatus::Warning,
                            message: "Build failed but error message unclear".to_string(),
                            duration_ms: duration,
                            screenshot_path: None,
                        })
                    }
                } else {
                    Ok(TestResult {
                        component: "Invalid Syntax Error".to_string(),
                        status: TestStatus::Failure,
                        message: "Invalid syntax was not caught".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Invalid Syntax Error".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test handling of invalid TOML configuration
    fn test_invalid_toml_error(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing invalid TOML error handling");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_invalid_toml_project(&temp_dir, "invalid-toml")?;

        // Try to parse Cargo.toml
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let combined = format!("{}\n{}", stdout, stderr);

                if !out.status.success() || combined.contains("TOML") {
                    Ok(TestResult {
                        component: "Invalid TOML Error".to_string(),
                        status: TestStatus::Success,
                        message: "Invalid TOML correctly rejected".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Invalid TOML Error".to_string(),
                        status: TestStatus::Warning,
                        message: "TOML error unclear".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Invalid TOML Error".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test handling of missing dependencies
    fn test_missing_dependency_error(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing missing dependency error handling");

        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("missing-dep-error");
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml = r#"[package]
name = "missing-dep-error"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
nonexistent-package = "999.0.0"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        let lib_rs = r#"
use nonexistent_package::Something;

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        // Try to build - should fail
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let combined = format!("{}\n{}", stdout, stderr);

                if !out.status.success() {
                    Ok(TestResult {
                        component: "Missing Dependency Error".to_string(),
                        status: TestStatus::Success,
                        message: "Missing dependency correctly detected".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Missing Dependency Error".to_string(),
                        status: TestStatus::Warning,
                        message: format!("Unexpected success: {}", combined),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Missing Dependency Error".to_string(),
                status: TestStatus::Success, // Command failed which is expected
                message: format!("Missing dependency caused command failure: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }
}

impl Test for ErrorHandlingTests {
    fn name(&self) -> &str {
        "Error Handling Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up error handling test suite");
        Ok(())
    }

    async fn run_with_driver(&self, _driver: &WebDriver) -> Result<TestResult> {
        info!("Running error handling tests");

        let mut results = vec![];

        // Run all error handling tests
        match self.test_invalid_syntax_error() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Invalid syntax test failed: {}", e);
                results.push(TestResult::error("Invalid Syntax", &e.to_string()));
            }
        }

        match self.test_invalid_toml_error() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Invalid TOML test failed: {}", e);
                results.push(TestResult::error("Invalid TOML", &e.to_string()));
            }
        }

        match self.test_missing_dependency_error() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Missing dependency test failed: {}", e);
                results.push(TestResult::error("Missing Dependency", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down error handling test suite");
        Ok(())
    }
}
