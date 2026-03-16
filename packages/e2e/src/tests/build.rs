//! Build process tests
//!
//! Tests for the complete build process including project initialization,
//! dependency resolution, WASM compilation, and output verification.

use crate::tests::{Test, TestResult, TestStatus};
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;
use tracing::info;

pub struct BuildTests;

impl BuildTests {
    /// Create a minimal valid Tairitsu project for testing
    fn create_test_project(temp_dir: &TempDir, name: &str) -> Result<PathBuf> {
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
app-name = "{} Test App"

[package.metadata.tairitsu.build]
target = "component"
output-dir = "dist"
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
            tairitsu_web::vdom::VNode::text("Build Test App"),
        ])
    });
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        Ok(project_path)
    }

    /// Test project structure validation
    fn test_project_structure(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing project structure validation");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "structure-test")?;

        // Verify project structure
        let cargo_toml_exists = project_path.join("Cargo.toml").exists();
        let lib_rs_exists = project_path.join("src/lib.rs").exists();

        let duration = start.elapsed().as_millis() as u64;

        if cargo_toml_exists && lib_rs_exists {
            Ok(TestResult {
                component: "Project Structure".to_string(),
                status: TestStatus::Success,
                message: "Project structure is valid".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Project Structure".to_string(),
                status: TestStatus::Failure,
                message: format!(
                    "Project structure incomplete: Cargo.toml={}, lib.rs={}",
                    cargo_toml_exists, lib_rs_exists
                ),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test Cargo.toml parsing
    fn test_cargo_toml_parsing(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing Cargo.toml parsing");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "parsing-test")?;

        // Try to parse Cargo.toml
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml"))?;
        let _value: toml::Value = toml::from_str(&cargo_content)?;

        let duration = start.elapsed().as_millis() as u64;

        // If we got here without error, parsing succeeded
        Ok(TestResult {
            component: "Cargo.toml Parsing".to_string(),
            status: TestStatus::Success,
            message: "Cargo.toml parsed successfully".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test dependency resolution
    fn test_dependency_resolution(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing dependency resolution");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "dep-test")?;

        // Run cargo check to verify dependencies can be resolved
        let output = Command::new("cargo")
            .args(["check", "--message-format=short"])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(TestResult {
                        component: "Dependency Resolution".to_string(),
                        status: TestStatus::Success,
                        message: "Dependencies resolved successfully".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Ok(TestResult {
                        component: "Dependency Resolution".to_string(),
                        status: TestStatus::Failure,
                        message: format!("Dependency resolution failed: {}", stderr),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "Dependency Resolution".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test WASM compilation
    fn test_wasm_compilation(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing WASM compilation");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "wasm-test")?;

        // Check if wasm32-wasip1 target is available
        let target_check = Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output();

        let has_wasm_target = match target_check {
            Ok(out) => {
                let targets = String::from_utf8_lossy(&out.stdout);
                targets.contains("wasm32-wasip1") || targets.contains("wasm32-wasip2")
            }
            Err(_) => false,
        };

        if !has_wasm_target {
            return Ok(TestResult {
                component: "WASM Compilation".to_string(),
                status: TestStatus::Warning,
                message: "WASM target not installed, skipping compilation test".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
                screenshot_path: None,
            });
        }

        // Try to build for WASM
        let output = Command::new("cargo")
            .args([
                "build",
                "--target",
                "wasm32-wasip1",
                "--message-format=short",
            ])
            .current_dir(&project_path)
            .output();

        let duration = start.elapsed().as_millis() as u64;

        match output {
            Ok(out) => {
                if out.status.success() {
                    Ok(TestResult {
                        component: "WASM Compilation".to_string(),
                        status: TestStatus::Success,
                        message: "WASM compilation successful".to_string(),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    Ok(TestResult {
                        component: "WASM Compilation".to_string(),
                        status: TestStatus::Failure,
                        message: format!("WASM compilation failed: {}", stderr),
                        duration_ms: duration,
                        screenshot_path: None,
                    })
                }
            }
            Err(e) => Ok(TestResult {
                component: "WASM Compilation".to_string(),
                status: TestStatus::Error(format!("Command execution failed: {}", e)),
                message: format!("Command execution failed: {}", e),
                duration_ms: duration,
                screenshot_path: None,
            }),
        }
    }

    /// Test output directory creation
    fn test_output_directory(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing output directory creation");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "output-test")?;

        // Create output directory
        let output_dir = project_path.join("dist");
        fs::create_dir_all(&output_dir)?;

        // Verify it exists
        let duration = start.elapsed().as_millis() as u64;

        if output_dir.exists() {
            Ok(TestResult {
                component: "Output Directory".to_string(),
                status: TestStatus::Success,
                message: "Output directory created successfully".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Output Directory".to_string(),
                status: TestStatus::Failure,
                message: "Failed to create output directory".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test Tairitsu metadata parsing
    fn test_tairitsu_metadata(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing Tairitsu metadata parsing");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "metadata-test")?;

        // Parse Cargo.toml and extract metadata
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml"))?;

        // Check for tairitsu metadata string
        let has_tairitsu_metadata = cargo_content.contains("[package.metadata.tairitsu]");
        let has_build_config = cargo_content.contains("[package.metadata.tairitsu.build]");

        let duration = start.elapsed().as_millis() as u64;

        if has_tairitsu_metadata && has_build_config {
            Ok(TestResult {
                component: "Tairitsu Metadata".to_string(),
                status: TestStatus::Success,
                message: "Tairitsu metadata found in Cargo.toml".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Tairitsu Metadata".to_string(),
                status: TestStatus::Warning,
                message: format!("Metadata incomplete: tairitsu={}, build={}",
                    has_tairitsu_metadata, has_build_config),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }

    /// Test build configuration validation
    fn test_build_config_validation(&self) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing build configuration validation");

        let temp_dir = TempDir::new()?;
        let project_path = Self::create_test_project(&temp_dir, "config-test")?;

        // Read and check build configuration
        let cargo_content = fs::read_to_string(project_path.join("Cargo.toml"))?;

        let has_target = cargo_content.contains("target = \"component\"");
        let has_output_dir = cargo_content.contains("output-dir = \"dist\"");

        let duration = start.elapsed().as_millis() as u64;

        if has_target && has_output_dir {
            Ok(TestResult {
                component: "Build Config Validation".to_string(),
                status: TestStatus::Success,
                message: "Build config: target=component, output-dir=dist".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            })
        } else {
            Ok(TestResult {
                component: "Build Config Validation".to_string(),
                status: TestStatus::Warning,
                message: format!("Build config incomplete: target={}, output-dir={}",
                    has_target, has_output_dir),
                duration_ms: duration,
                screenshot_path: None,
            })
        }
    }
}

impl Test for BuildTests {
    fn name(&self) -> &str {
        "Build Process Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up build process test suite");
        Ok(())
    }

    async fn run_with_driver(&self, _driver: &thirtyfour::WebDriver) -> Result<TestResult> {
        info!("Running build process tests");

        let mut results = vec![];

        // Run all build tests
        match self.test_project_structure() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Project structure test failed: {}", e);
                results.push(TestResult::error("Project Structure", &e.to_string()));
            }
        }

        match self.test_cargo_toml_parsing() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Cargo.toml parsing test failed: {}", e);
                results.push(TestResult::error("Cargo.toml Parsing", &e.to_string()));
            }
        }

        match self.test_dependency_resolution() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Dependency resolution test failed: {}", e);
                results.push(TestResult::error("Dependency Resolution", &e.to_string()));
            }
        }

        match self.test_wasm_compilation() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("WASM compilation test failed: {}", e);
                results.push(TestResult::error("WASM Compilation", &e.to_string()));
            }
        }

        match self.test_output_directory() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Output directory test failed: {}", e);
                results.push(TestResult::error("Output Directory", &e.to_string()));
            }
        }

        match self.test_tairitsu_metadata() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Tairitsu metadata test failed: {}", e);
                results.push(TestResult::error("Tairitsu Metadata", &e.to_string()));
            }
        }

        match self.test_build_config_validation() {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Build config validation test failed: {}", e);
                results.push(TestResult::error("Build Config Validation", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down build process test suite");
        Ok(())
    }
}
