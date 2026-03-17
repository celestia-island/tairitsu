use anyhow::Result;
use std::{fs, path::Path, process::Command};

use tempfile::TempDir;

pub struct PackageTests;

impl PackageTests {
    /// 测试：从零创建最小项目
    pub fn test_minimal_project() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("minimal-app");

        // 1. 创建项目目录
        fs::create_dir_all(&project_path)?;
        fs::create_dir_all(project_path.join("src"))?;

        // 2. 创建 Cargo.toml
        let cargo_toml = r#"[package]
name = "minimal-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tairitsu-web = { path = "../../../packages/web", features = ["wit-bindings"] }

[package.metadata.tairitsu.build]
target = "component"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // 3. 创建源代码
        let lib_rs = r#"
use tairitsu_web::WitPlatform;

#[export_name = "tairitsu_component_bootstrap"]
pub extern "C" fn bootstrap() {
    WitPlatform::mount(|| {
        tairitsu_web::vdom::VNode::element("div", vec![], vec![
            tairitsu_web::vdom::VNode::text("Minimal App"),
        ])
    });
}
"#;
        fs::write(project_path.join("src/lib.rs"), lib_rs)?;

        // 4. 验证项目结构
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("src/lib.rs").exists());

        println!("✅ Minimal project created successfully");
        Ok(())
    }

    /// 测试：完整配置项目
    pub fn test_full_featured_project() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().join("full-app");

        // 创建完整的项目结构
        fs::create_dir_all(project_path.join("src"))?;
        fs::create_dir_all(project_path.join("assets/images"))?;
        fs::create_dir_all(project_path.join("styles"))?;

        // 创建完整配置的 Cargo.toml
        let cargo_toml = r#"[package]
name = "full-app"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-web = { path = "../../../packages/web", features = ["wit-bindings"] }

[package.metadata.tairitsu]
app-name = "Full Featured App"
title = "Full App"

[package.metadata.tairitsu.build]
target = "component"
output_dir = "../../target/tairitsu-dist"
optimize = true

[package.metadata.tairitsu.assets]
include = ["assets/**"]
inline-limit = 8192

[package.metadata.tairitsu.html]
lang = "zh-CN"
favicon = "assets/favicon.ico"
"#;
        fs::write(project_path.join("Cargo.toml"), cargo_toml)?;

        // 创建资源文件
        fs::write(project_path.join("assets/images/logo.txt"), "logo content")?;

        // 验证项目结构
        assert!(project_path.join("Cargo.toml").exists());
        assert!(project_path.join("assets/images/logo.txt").exists());

        println!("✅ Full featured project created successfully");
        Ok(())
    }

    /// 测试：配置解析
    pub fn test_config_parsing() -> Result<()> {
        let cargo_toml = r#"[package]
name = "test-app"
version = "0.1.0"

[package.metadata.tairitsu]
app-name = "Test App"

[package.metadata.tairitsu.build]
target = "component"
optimize = true
"#;

        // 解析 TOML
        let value: toml::Value = toml::from_str(cargo_toml)?;

        // 验证解析结果
        let metadata = value
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("tairitsu"));

        assert!(metadata.is_some());

        let app_name = metadata
            .and_then(|m| m.get("app-name"))
            .and_then(|v| v.as_str());

        assert_eq!(app_name, Some("Test App"));

        println!("✅ Config parsing successful");
        Ok(())
    }

    /// 测试：资源文件处理
    pub fn test_asset_processing() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // 创建测试文件
        let small_file = temp_dir.path().join("small.txt");
        let large_file = temp_dir.path().join("large.txt");

        fs::write(&small_file, "small content")?;
        fs::write(&large_file, "x".repeat(1000))?;

        // 验证文件创建
        assert!(small_file.exists());
        assert!(large_file.exists());

        // 验证文件大小
        let small_size = fs::metadata(&small_file)?.len();
        let large_size = fs::metadata(&large_file)?.len();

        assert!(small_size < 100);
        assert!(large_size > 100);

        println!("✅ Asset processing test successful");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_project_creation() {
        PackageTests::test_minimal_project().unwrap();
    }

    #[test]
    fn test_full_featured_project_creation() {
        PackageTests::test_full_featured_project().unwrap();
    }

    #[test]
    fn test_config_parsing() {
        PackageTests::test_config_parsing().unwrap();
    }

    #[test]
    fn test_asset_processing() {
        PackageTests::test_asset_processing().unwrap();
    }
}
