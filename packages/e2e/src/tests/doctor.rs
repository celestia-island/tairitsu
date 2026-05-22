//! Doctor command tests
//!
//! Tests for the `tairitsu doctor` command which checks project compatibility
//! and environment setup.

use anyhow::Result;
use std::{fs, path::PathBuf, process::Command, time::Instant};

use tempfile::TempDir;
use thirtyfour::WebDriver;
use tracing::info;

use crate::tests::{Test, TestResult, TestStatus};

pub struct DoctorTests;

impl DoctorTests {
    /// Create a minimal valid Tairitsu project
    fn create_minimal_project(temp_dir: &TempDir, name: &str) -> Result<PathBuf> {
        let project_path = temp_dir.path().join(name);
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-web = {{ path = "../../../packages/web", features = ["wit-bindings"] }}

[package.metadata.tairitsu]
app-name = "{}"

[package.metadata.tairitsu.build]
target = "component"
"#,
            name, name
        );
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        let lib_rs = r#"
use tairitsu_web::WitPlatform;

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    WitPlatform::mount(|| {
        tairitsu_web::vdom::VNode::element("div", vec![], vec![
            tairitsu_web::vdom::VNode::text("Hello, World!"),
        ])
    });
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Test doctor on a valid project
    fn test_valid_project(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing doctor on valid project");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_minimal_project(&temp_dir, "valid-project")?;

        // Run tairitsu doctor command
        let output = Command::new("cargo")
            .args(["run", "--package", "tairitsu-packager", "--", "doctor"])
            .current_dir(&project_path)
            .output()?;

        let duration = start.elapsed().as_millis() as u64;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(TestResult {
                component: "Doctor (Valid Project)".to_string(),
                status: TestStatus::Success,
                message: format!("Doctor passed for valid project: {}", stdout.trim()),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Doctor (Valid Project)".to_string(),
                status: TestStatus::Failure,
                message: format!("Doctor failed: {}", stderr),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test doctor on a project without Cargo.toml
    fn test_missing_cargo_toml(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing doctor with missing Cargo.toml");

        let temp_dir = TempDir::new()?;
        let empty_path = temp_dir.path().join("empty-project");
        fs::create_dir_all(&empty_path)?;

        // Run tairitsu doctor command
        let output = Command::new("cargo")
            .args(["run", "--package", "tairitsu-packager", "--", "doctor"])
            .current_dir(&empty_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                if !out.status.success() && stderr.contains("Cargo.toml") {
                    Ok(TestResult {
                        component: "Doctor (Missing Cargo.toml)".to_string(),
                        status: TestStatus::Success,
                        message: "Doctor correctly identified missing Cargo.toml".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Doctor (Missing Cargo.toml)".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Expected failure but got: {}", stderr),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Doctor (Missing Cargo.toml)".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test doctor on a project with missing tairitsu-web dependency
    fn test_missing_dependency(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing doctor with missing tairitsu-web dependency");

        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("missing-dep");
        fs::create_dir_all(project_path.join("src"))?;

        let cargo_toml = r#"[package]
name = "missing-dep"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
# Missing tairitsu-web dependency
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        let lib_rs = r#"
#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    // Empty bootstrap
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        // Run tairitsu doctor command
        let output = Command::new("cargo")
            .args(["run", "--package", "tairitsu-packager", "--", "doctor"])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let combined = format!("{}\n{}", stdout, stderr);

                if combined.contains("tairitsu-web") && combined.contains("dependencies") {
                    Ok(TestResult {
                        component: "Doctor (Missing Dependency)".to_string(),
                        status: TestStatus::Success,
                        message: "Doctor correctly identified missing tairitsu-web".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Doctor (Missing Dependency)".to_string(),
                        status: TestStatus::Failure,
                        message: format!(
                            "Doctor did not identify missing dependency. Output: {}",
                            combined
                        ),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Doctor (Missing Dependency)".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test doctor JSON output format
    fn test_json_output(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing doctor JSON output format");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_minimal_project(&temp_dir, "json-test")?;

        // Run tairitsu doctor command with JSON output
        let output = Command::new("cargo")
            .args([
                "run",
                "--package",
                "tairitsu-packager",
                "--",
                "doctor",
                "--format",
                "json",
            ])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);

                // Try to parse as JSON
                if let Ok(_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    Ok(TestResult {
                        component: "Doctor (JSON Format)".to_string(),
                        status: TestStatus::Success,
                        message: "Doctor JSON output is valid".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    Ok(TestResult {
                        component: "Doctor (JSON Format)".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Doctor JSON output is invalid: {}", stdout),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Doctor (JSON Format)".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }
}

impl Test for DoctorTests {
    fn name(&self) -> &str {
        "Doctor Command Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up doctor command test suite");
        Ok(())
    }

    async fn run_with_driver(&self, _driver: &WebDriver) -> Result<TestResult> {
        info!("Running doctor command tests");

        let mut results = vec![];

        // Run all doctor tests
        match self.test_valid_project() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Valid project test failed: {}", e);
                results.push(TestResult::error("Doctor (Valid Project)", &e.to_string()));
            }
        }

        match self.test_missing_cargo_toml() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Missing Cargo.toml test failed: {}", e);
                results.push(TestResult::error(
                    "Doctor (Missing Cargo.toml)",
                    &e.to_string(),
                ));
            }
        }

        match self.test_missing_dependency() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Missing dependency test failed: {}", e);
                results.push(TestResult::error(
                    "Doctor (Missing Dependency)",
                    &e.to_string(),
                ));
            }
        }

        match self.test_json_output() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("JSON output test failed: {}", e);
                results.push(TestResult::error("Doctor (JSON Format)", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down doctor command test suite");
        Ok(())
    }
}
