//! Doctor module for checking project compatibility and environment setup
//!
//! This module provides comprehensive diagnostics for Tairitsu projects including:
//! - Project dependency checks (Dioxus, wasm-bindgen, etc.)
//! - Environment checks (wasm32-wasip2 target installation)
//! - Configuration checks (package.metadata.tairitsu validity)
//! - Migration suggestions for legacy projects

use crate::{Result, TairitsuPackagerError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Severity level for diagnostic messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
}

/// A diagnostic message with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub category: DiagnosticCategory,
    pub message: String,
    pub suggestion: Option<String>,
    pub help_url: Option<String>,
}

/// Category of diagnostic message
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticCategory {
    Dependencies,
    Environment,
    Configuration,
    Build,
    Migration,
}

/// Result of a doctor check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub project_path: PathBuf,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: DoctorSummary,
}

/// Summary of diagnostic results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorSummary {
    pub total_checks: usize,
    pub passed: usize,
    pub warnings: usize,
    pub errors: usize,
}

impl DoctorSummary {
    pub fn is_healthy(&self) -> bool {
        self.errors == 0
    }
}

/// Main doctor checker
pub struct DoctorChecker {
    project_path: PathBuf,
    cargo_content: Option<String>,
    manifest: Option<toml::Value>,
}

impl DoctorChecker {
    /// Create a new doctor checker for the given project path
    pub fn new(project_path: &Path) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            cargo_content: None,
            manifest: None,
        }
    }

    /// Run all diagnostic checks
    pub fn check_all(&mut self) -> Result<DoctorReport> {
        // Load Cargo.toml
        self.load_cargo_toml()?;

        let mut diagnostics = Vec::new();

        // Run checks in order
        diagnostics.extend(self.check_project_structure()?);
        diagnostics.extend(self.check_dependencies()?);
        diagnostics.extend(self.check_environment()?);
        diagnostics.extend(self.check_tairitsu_config()?);
        diagnostics.extend(self.check_migration_needed()?);

        let summary = self.create_summary(&diagnostics);

        Ok(DoctorReport {
            project_path: self.project_path.clone(),
            diagnostics,
            summary,
        })
    }

    /// Load and parse Cargo.toml
    fn load_cargo_toml(&mut self) -> Result<()> {
        let cargo_path = self.project_path.join("Cargo.toml");

        if !cargo_path.exists() {
            return Err(TairitsuPackagerError::ConfigNotFound(
                cargo_path.display().to_string(),
            ));
        }

        self.cargo_content = Some(std::fs::read_to_string(&cargo_path)?);
        let content = self.cargo_content.as_ref().unwrap();

        let manifest: toml::Value = toml::from_str(content)
            .map_err(|e| TairitsuPackagerError::InvalidConfig(format!("Invalid TOML: {}", e)))?;

        self.manifest = Some(manifest);
        Ok(())
    }

    /// Check basic project structure
    fn check_project_structure(&self) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Check for src directory
        let src_dir = self.project_path.join("src");
        if !src_dir.exists() {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                category: DiagnosticCategory::Configuration,
                message: "src directory not found".to_string(),
                suggestion: Some("Create a src directory with your Rust source files".to_string()),
                help_url: None,
            });
        }

        // Check for src/lib.rs
        let lib_rs = src_dir.join("lib.rs");
        if !lib_rs.exists() {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Configuration,
                message: "src/lib.rs not found".to_string(),
                suggestion: Some("Ensure your project has a library entry point".to_string()),
                help_url: None,
            });
        }

        Ok(diagnostics)
    }

    /// Check project dependencies
    fn check_dependencies(&self) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        let manifest = self.manifest.as_ref().unwrap();

        // Get dependencies section
        let dependencies = manifest.get("dependencies");

        if dependencies.is_none() {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                category: DiagnosticCategory::Dependencies,
                message: "No dependencies found in Cargo.toml".to_string(),
                suggestion: Some("Add tairitsu-web as a dependency".to_string()),
                help_url: Some("https://docs.tairitsu.dev/getting-started".to_string()),
            });
            return Ok(diagnostics);
        }

        let deps = dependencies.unwrap().as_table();

        if deps.is_none() {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                category: DiagnosticCategory::Dependencies,
                message: "Dependencies section is not a table".to_string(),
                suggestion: None,
                help_url: None,
            });
            return Ok(diagnostics);
        }

        let deps_table = deps.unwrap();
        let dep_names: HashSet<String> = deps_table.keys().cloned().collect();

        // Check for tairitsu-web
        if !dep_names.contains("tairitsu-web") {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                category: DiagnosticCategory::Dependencies,
                message: "tairitsu-web dependency not found".to_string(),
                suggestion: Some(
                    "Add tairitsu-web = { version = \"0.1\", features = [\"wit-bindings\"] } to dependencies".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/getting-started".to_string()),
            });
        } else {
            // Check tairitsu-web version
            if let Some(dep_value) = deps_table.get("tairitsu-web") {
                if let Some(table) = dep_value.as_table() {
                    if table.contains_key("path") {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Info,
                            category: DiagnosticCategory::Dependencies,
                            message: "Using local path for tairitsu-web".to_string(),
                            suggestion: None,
                            help_url: None,
                        });
                    }
                }
            }
        }

        // Check for conflicting dependencies (Dioxus)
        if dep_names.contains("dioxus") || dep_names.contains("dioxus-web") {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Dependencies,
                message: "Dioxus dependencies detected".to_string(),
                suggestion: Some(
                    "Tairitsu uses its own component model. Consider removing Dioxus dependencies if migrating.".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/migration/from-dioxus".to_string()),
            });
        }

        // Check for wasm-bindgen (legacy)
        if dep_names.contains("wasm-bindgen") {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Dependencies,
                message: "wasm-bindgen dependency detected (legacy WASM)".to_string(),
                suggestion: Some(
                    "Tairitsu uses component-model WASM. wasm-bindgen is not required.".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/migration/from-wasm-bindgen".to_string()),
            });
        }

        // Check for leptos (another framework)
        if dep_names.contains("leptos") {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Dependencies,
                message: "Leptos dependencies detected".to_string(),
                suggestion: Some(
                    "Tairitsu uses its own component model. Consider removing Leptos dependencies if migrating.".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/migration/from-leptos".to_string()),
            });
        }

        // Check crate type
        if let Some(lib) = manifest.get("lib") {
            if let Some(crate_types) = lib.get("crate-type") {
                if let Some(types) = crate_types.as_array() {
                    let has_cdylib = types.iter().any(|v| {
                        v.as_str().map(|s| s == "cdylib").unwrap_or(false)
                    });

                    if !has_cdylib {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            category: DiagnosticCategory::Configuration,
                            message: "[lib] crate-type does not include \"cdylib\"".to_string(),
                            suggestion: Some(
                                "Add crate-type = [\"cdylib\", \"rlib\"] to [lib] section".to_string(),
                            ),
                            help_url: None,
                        });
                    }
                }
            }
        } else {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Configuration,
                message: "[lib] section not found in Cargo.toml".to_string(),
                suggestion: Some(
                    "Add [lib] section with crate-type = [\"cdylib\", \"rlib\"]".to_string(),
                ),
                help_url: None,
            });
        }

        Ok(diagnostics)
    }

    /// Check environment setup
    fn check_environment(&self) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        // Check for Rust installation
        let rust_version_output = Command::new("rustc")
            .arg("--version")
            .output();

        match rust_version_output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                diagnostics.push(Diagnostic {
                    severity: Severity::Info,
                    category: DiagnosticCategory::Environment,
                    message: format!("Rust toolchain installed: {}", version.trim()),
                    suggestion: None,
                    help_url: None,
                });
            }
            _ => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    category: DiagnosticCategory::Environment,
                    message: "Rust toolchain not found".to_string(),
                    suggestion: Some("Install Rust from https://rustup.rs".to_string()),
                    help_url: Some("https://rustup.rs".to_string()),
                });
                return Ok(diagnostics); // Skip further checks if Rust is not installed
            }
        }

        // Check for wasm32-wasip2 target
        let target_output = Command::new("rustup")
            .args(["target", "list", "--installed"])
            .output();

        match target_output {
            Ok(output) if output.status.success() => {
                let targets = String::from_utf8_lossy(&output.stdout);
                let has_wasip2 = targets.contains("wasm32-wasip1") || targets.contains("wasm32-wasip2");

                if !has_wasip2 {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        category: DiagnosticCategory::Environment,
                        message: "wasm32-wasip2 target not installed".to_string(),
                        suggestion: Some("Run: rustup target add wasm32-wasip1".to_string()),
                        help_url: Some("https://docs.tairitsu.dev/setup/wasm-target".to_string()),
                    });
                } else {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Info,
                        category: DiagnosticCategory::Environment,
                        message: "WASM target installed".to_string(),
                        suggestion: None,
                        help_url: None,
                    });
                }
            }
            _ => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    category: DiagnosticCategory::Environment,
                    message: "Could not check installed Rust targets".to_string(),
                    suggestion: Some("Ensure rustup is properly installed".to_string()),
                    help_url: None,
                });
            }
        }

        // Check for cargo component (optional but recommended)
        let component_output = Command::new("cargo")
            .args(["component", "--version"])
            .output();

        match component_output {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout);
                diagnostics.push(Diagnostic {
                    severity: Severity::Info,
                    category: DiagnosticCategory::Environment,
                    message: format!("cargo-component installed: {}", version.trim()),
                    suggestion: None,
                    help_url: None,
                });
            }
            _ => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Info,
                    category: DiagnosticCategory::Environment,
                    message: "cargo-component not found (optional)".to_string(),
                    suggestion: Some("Install with: cargo install cargo-component".to_string()),
                    help_url: Some("https://github.com/bytecodealliance/cargo-component".to_string()),
                });
            }
        }

        Ok(diagnostics)
    }

    /// Check Tairitsu configuration
    fn check_tairitsu_config(&self) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        let manifest = self.manifest.as_ref().unwrap();

        // Check for package.metadata.tairitsu
        let has_metadata = manifest
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("tairitsu"))
            .is_some();

        if !has_metadata {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Configuration,
                message: "[package.metadata.tairitsu] section not found".to_string(),
                suggestion: Some(
                    "Add [package.metadata.tairitsu] section to Cargo.toml for configuration".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/configuration".to_string()),
            });
            return Ok(diagnostics);
        }

        // Validate metadata structure
        let metadata = manifest
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("tairitsu"))
            .unwrap();

        // Check build target
        if let Some(build) = metadata.get("build") {
            if let Some(target) = build.get("target") {
                if let Some(target_str) = target.as_str() {
                    if target_str == "component" {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Info,
                            category: DiagnosticCategory::Configuration,
                            message: "Build target is set to 'component'".to_string(),
                            suggestion: None,
                            help_url: None,
                        });
                    } else {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            category: DiagnosticCategory::Configuration,
                            message: format!("Build target is '{}' (expected 'component')", target_str),
                            suggestion: Some("Set target to 'component' for WASM component builds".to_string()),
                            help_url: None,
                        });
                    }
                }
            }
        }

        // Check dev configuration
        if metadata.get("dev").is_some() {
            diagnostics.push(Diagnostic {
                severity: Severity::Info,
                category: DiagnosticCategory::Configuration,
                message: "Dev configuration found".to_string(),
                suggestion: None,
                help_url: None,
            });
        }

        // Check assets configuration
        if metadata.get("assets").is_some() {
            diagnostics.push(Diagnostic {
                severity: Severity::Info,
                category: DiagnosticCategory::Configuration,
                message: "Assets configuration found".to_string(),
                suggestion: None,
                help_url: None,
            });
        }

        Ok(diagnostics)
    }

    /// Check if migration is needed from legacy frameworks
    fn check_migration_needed(&self) -> Result<Vec<Diagnostic>> {
        let mut diagnostics = Vec::new();

        let manifest = self.manifest.as_ref().unwrap();

        // Check for legacy Dioxus patterns
        let dependencies = manifest.get("dependencies").and_then(|d| d.as_table());
        let has_dioxus = dependencies
            .map(|d| d.contains_key("dioxus") || d.contains_key("dioxus-web"))
            .unwrap_or(false);

        if has_dioxus {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Migration,
                message: "Migration from Dioxus may be needed".to_string(),
                suggestion: Some(
                    "See migration guide: https://docs.tairitsu.dev/migration/from-dioxus".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/migration/from-dioxus".to_string()),
            });
        }

        // Check for legacy wasm-bindgen patterns
        let has_wasm_bindgen = dependencies
            .map(|d| d.contains_key("wasm-bindgen"))
            .unwrap_or(false);

        if has_wasm_bindgen {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                category: DiagnosticCategory::Migration,
                message: "Migration from wasm-bindgen may be needed".to_string(),
                suggestion: Some(
                    "See migration guide: https://docs.tairitsu.dev/migration/from-wasm-bindgen".to_string(),
                ),
                help_url: Some("https://docs.tairitsu.dev/migration/from-wasm-bindgen".to_string()),
            });
        }

        // Check for legacy build scripts
        let build_rs = self.project_path.join("build.rs");
        if build_rs.exists() {
            let build_content = std::fs::read_to_string(&build_rs).unwrap_or_default();
            if build_content.contains("wasm-bindgen") || build_content.contains("wasm-pack") {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    category: DiagnosticCategory::Migration,
                    message: "build.rs contains legacy WASM build references".to_string(),
                    suggestion: Some("Consider updating build.rs for Tairitsu component model".to_string()),
                    help_url: Some("https://docs.tairitsu.dev/migration/build-scripts".to_string()),
                });
            }
        }

        Ok(diagnostics)
    }

    /// Create summary from diagnostics
    fn create_summary(&self, diagnostics: &[Diagnostic]) -> DoctorSummary {
        let total_checks = diagnostics.len();
        let errors = diagnostics.iter().filter(|d| d.severity == Severity::Error).count();
        let warnings = diagnostics.iter().filter(|d| d.severity == Severity::Warning).count();
        let passed = total_checks - errors - warnings;

        DoctorSummary {
            total_checks,
            passed,
            warnings,
            errors,
        }
    }
}

/// Format diagnostic as a human-readable string
pub fn format_diagnostic(diagnostic: &Diagnostic) -> String {
    let icon = match diagnostic.severity {
        Severity::Info => "ℹ",
        Severity::Warning => "⚠",
        Severity::Error => "✖",
    };

    let category = match diagnostic.category {
        DiagnosticCategory::Dependencies => "dependencies",
        DiagnosticCategory::Environment => "environment",
        DiagnosticCategory::Configuration => "configuration",
        DiagnosticCategory::Build => "build",
        DiagnosticCategory::Migration => "migration",
    };

    let mut result = format!("{} [{category}] {}", icon, diagnostic.message);

    if let Some(suggestion) = &diagnostic.suggestion {
        result.push_str(&format!("\n   → {}", suggestion));
    }

    if let Some(help_url) = &diagnostic.help_url {
        result.push_str(&format!("\n   → Learn more: {}", help_url));
    }

    result
}

/// Format the full doctor report
pub fn format_report(report: &DoctorReport) -> String {
    let mut output = String::new();

    output.push_str(&format!(
        "\n🩺 Tairitsu Doctor Report for: {}\n",
        report.project_path.display()
    ));
    output.push_str(&format!("{}\n", "=".repeat(60)));

    // Group diagnostics by severity
    let errors: Vec<_> = report.diagnostics.iter().filter(|d| d.severity == Severity::Error).collect();
    let warnings: Vec<_> = report.diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect();
    let info: Vec<_> = report.diagnostics.iter().filter(|d| d.severity == Severity::Info).collect();

    if !errors.is_empty() {
        output.push_str(&format!("\n🔴 ERRORS ({}):\n", errors.len()));
        for diagnostic in errors {
            output.push_str(&format!("\n{}\n", format_diagnostic(diagnostic)));
        }
    }

    if !warnings.is_empty() {
        output.push_str(&format!("\n🟡 WARNINGS ({}):\n", warnings.len()));
        for diagnostic in warnings {
            output.push_str(&format!("\n{}\n", format_diagnostic(diagnostic)));
        }
    }

    if !info.is_empty() {
        output.push_str(&format!("\n🔵 INFO ({}):\n", info.len()));
        for diagnostic in info {
            output.push_str(&format!("\n{}\n", format_diagnostic(diagnostic)));
        }
    }

    // Summary
    output.push_str(&format!("\n{}\n", "=".repeat(60)));
    output.push_str(&format!(
        "Summary: {} checks passed, {} warnings, {} errors\n",
        report.summary.passed, report.summary.warnings, report.summary.errors
    ));

    if report.summary.is_healthy() {
        output.push_str("✅ Project is healthy!\n");
    } else {
        output.push_str("❌ Project has issues that need attention.\n");
    }

    output
}

/// Run doctor check on the given project path
pub fn run_doctor(project_path: &Path) -> Result<DoctorReport> {
    let mut checker = DoctorChecker::new(project_path);
    checker.check_all()
}

/// Quick check: verify if project is compatible with Tairitsu
pub fn is_compatible(project_path: &Path) -> bool {
    let cargo_path = project_path.join("Cargo.toml");
    if !cargo_path.exists() {
        return false;
    }

    let content = match std::fs::read_to_string(&cargo_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let manifest: toml::Value = match toml::from_str(&content) {
        Ok(m) => m,
        Err(_) => return false,
    };

    // Check for tairitsu-web dependency
    let has_tairitsu_web = manifest
        .get("dependencies")
        .and_then(|d| d.as_table())
        .map(|d| d.contains_key("tairitsu-web"))
        .unwrap_or(false);

    has_tairitsu_web
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_severity_ordering() {
        assert!(Severity::Error > Severity::Warning);
        assert!(Severity::Warning > Severity::Info);
    }

    #[test]
    fn test_summary_healthy() {
        let summary = DoctorSummary {
            total_checks: 5,
            passed: 5,
            warnings: 0,
            errors: 0,
        };
        assert!(summary.is_healthy());

        let summary_unhealthy = DoctorSummary {
            total_checks: 5,
            passed: 2,
            warnings: 2,
            errors: 1,
        };
        assert!(!summary_unhealthy.is_healthy());
    }
}
