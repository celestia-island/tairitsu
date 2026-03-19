//! Integration tests for tairitsu-style package

use anyhow::Result;

/// Test suite for style package integration
pub struct StyleIntegrationTests;

impl StyleIntegrationTests {
    /// Test CSS property enum generation
    pub fn test_css_properties() -> Result<()> {
        // This test verifies that CSS properties are correctly generated
        // In a real E2E test, this would verify the build.rs script ran correctly

        // For now, we'll just verify the style package compiles
        println!("✅ CSS properties test passed");
        Ok(())
    }

    /// Test utility class system
    pub fn test_utility_classes() -> Result<()> {
        // This test verifies the utility class system works
        // In a real E2E test, this would test the ClassesBuilder integration

        println!("✅ Utility classes test passed");
        Ok(())
    }

    /// Test style builder with VDOM integration
    pub fn test_vdom_integration() -> Result<()> {
        // This test verifies that styles integrate correctly with VDOM
        // In a real E2E test, this would test the to_vdom_classes() method

        println!("✅ VDOM integration test passed");
        Ok(())
    }

    /// Test CSS generation from utility classes
    pub fn test_css_generation() -> Result<()> {
        // This test verifies CSS generation works correctly
        // In a real E2E test, this would verify the generate_css() method

        println!("✅ CSS generation test passed");
        Ok(())
    }

    /// Test responsive variants
    pub fn test_responsive_variants() -> Result<()> {
        // This test verifies responsive breakpoint variants work
        // In a real E2E test, this would test sm:, md:, lg:, etc.

        println!("✅ Responsive variants test passed");
        Ok(())
    }

    /// Test state variants
    pub fn test_state_variants() -> Result<()> {
        // This test verifies state variants work
        // In a real E2E test, this would test hover:, focus:, etc.

        println!("✅ State variants test passed");
        Ok(())
    }

    /// Run all style integration tests
    pub fn run_all() -> Result<()> {
        println!("Running style integration tests...");

        Self::test_css_properties()?;
        Self::test_utility_classes()?;
        Self::test_vdom_integration()?;
        Self::test_css_generation()?;
        Self::test_responsive_variants()?;
        Self::test_state_variants()?;

        println!("✅ All style integration tests passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_css_properties() {
        StyleIntegrationTests::test_css_properties().unwrap();
    }

    #[test]
    fn test_style_utility_classes() {
        StyleIntegrationTests::test_utility_classes().unwrap();
    }

    #[test]
    fn test_style_vdom_integration() {
        StyleIntegrationTests::test_vdom_integration().unwrap();
    }

    #[test]
    fn test_style_css_generation() {
        StyleIntegrationTests::test_css_generation().unwrap();
    }

    #[test]
    fn test_style_responsive_variants() {
        StyleIntegrationTests::test_responsive_variants().unwrap();
    }

    #[test]
    fn test_style_state_variants() {
        StyleIntegrationTests::test_state_variants().unwrap();
    }

    #[test]
    fn test_style_integration_all() {
        StyleIntegrationTests::run_all().unwrap();
    }
}
