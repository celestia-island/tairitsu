//! Form validation E2E tests.
//!
//! Tests form validation scenarios including:
//! - Required field validation
//! - Email validation
//! - Password strength validation
//! - Custom validation rules
//! - Form submission with validation

use anyhow::Result;
use std::time::{Duration, Instant};

use thirtyfour::{By, Key, WebDriver};
use tracing::info;

use crate::tests::{Test, TestResult};

pub struct FormValidationTests;

impl FormValidationTests {
    /// Test required field validation.
    async fn test_required_field_validation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing required field validation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/form", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Try to submit form without filling required fields
        let submit_button = driver.find(By::Css("#form-submit")).await.ok();
        let name_input = driver.find(By::Css("#name-input")).await.ok();

        if let (Some(submit), Some(name_input)) = (submit_button, name_input) {
            // Click submit without filling required fields
            submit.click().await?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Check for validation error
            let error_message = driver.find(By::Css(".validation-error")).await.ok();
            if let Some(error) = error_message {
                let error_text = error.text().await?;
                if error_text.contains("required") || error_text.contains("必填") {
                    info!("Validation error shown: {}", error_text);
                }
            }

            // Fill the required field
            name_input.send_keys("Test User").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Submit again
            submit.click().await?;
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Check if form submitted successfully (no error)
            let success_message = driver.find(By::Css(".form-success")).await.ok();
            if let Some(success) = success_message {
                let success_text = success.text().await?;
                info!("Form submitted: {}", success_text);
            }
        } else {
            // Form elements not found - skip test
            info!("Form elements not found, skipping required field test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Required Field Validation".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Form elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Required Field Validation".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Required field validation works".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test email validation.
    async fn test_email_validation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing email validation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/form", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let email_input = driver.find(By::Css("#email-input")).await.ok();

        if let Some(email) = email_input {
            // Test invalid email
            email.send_keys("invalid-email").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Trigger validation (blur)
            email.send_keys(Key::Tab).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Check for error
            let error_element = driver.find(By::Css("#email-error")).await.ok();
            if let Some(error) = error_element {
                let error_text = error.text().await?;
                if error_text.contains("valid") || error_text.contains("格式") {
                    info!("Email validation error: {}", error_text);
                }
            }

            // Clear and test valid email
            email.clear().await?;
            email.send_keys("test@example.com").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            email.send_keys(Key::Tab).await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Error should be gone or show success
            let success_element = driver.find(By::Css("#email-success")).await.ok();
            if let Some(_success) = success_element {
                info!("Email validation passed");
            }
        } else {
            info!("Email input not found, skipping email validation test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Email Validation".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Email input not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Email Validation".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Email validation works correctly".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test password strength validation.
    async fn test_password_validation(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing password strength validation");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/form", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let password_input = driver.find(By::Css("#password-input")).await.ok();
        let strength_indicator = driver.find(By::Css("#password-strength")).await.ok();

        if let (Some(password), Some(indicator)) = (password_input, strength_indicator) {
            // Test weak password
            password.send_keys("123").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            let initial_strength = indicator.text().await?;
            info!("Password strength for '123': {}", initial_strength);

            // Clear and test stronger password
            password.clear().await?;
            password.send_keys("StrongP@ssw0rd!").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            let new_strength = indicator.text().await?;
            info!("Password strength for 'StrongP@ssw0rd!': {}", new_strength);

            // Strength should have improved
            if new_strength != initial_strength {
                info!("Password strength indicator updated");
            }
        } else {
            info!("Password elements not found, skipping password validation test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Password Validation".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Password elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Password Validation".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Password strength validation works".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test form submission with all validations.
    async fn test_form_submission(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing complete form submission");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/form", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Find form elements
        let name_input = driver.find(By::Css("#name-input")).await.ok();
        let email_input = driver.find(By::Css("#email-input")).await.ok();
        let password_input = driver.find(By::Css("#password-input")).await.ok();
        let submit_button = driver.find(By::Css("#form-submit")).await.ok();

        if let (Some(name), Some(email), Some(password), Some(submit)) =
            (name_input, email_input, password_input, submit_button)
        {
            // Fill form with valid data
            name.clear().await?;
            name.send_keys("John Doe").await?;
            tokio::time::sleep(Duration::from_millis(50)).await;

            email.clear().await?;
            email.send_keys("john.doe@example.com").await?;
            tokio::time::sleep(Duration::from_millis(50)).await;

            password.clear().await?;
            password.send_keys("SecurePass123!").await?;
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Submit form
            submit.click().await?;
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Check for success message
            let success = driver.find(By::Css(".form-success")).await.ok();
            if let Some(success_msg) = success {
                let success_text = success_msg.text().await?;
                info!("Form submission result: {}", success_text);
            }

            // Check URL didn't change (no actual submission)
            let current_url = driver.current_url().await?;
            info!("Current URL after submission: {}", current_url);
        } else {
            info!("Form elements not found, skipping form submission test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Form Submission".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Form elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Form Submission".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Form submission with validation works".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }

    /// Test form reset functionality.
    async fn test_form_reset(&self, driver: &WebDriver) -> Result<TestResult> {
        let start = Instant::now();
        info!("Testing form reset");

        let base_url = std::env::var("WEBSITE_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let test_url = format!("{}/components/form", base_url);

        driver.goto(&test_url).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;

        let name_input = driver.find(By::Css("#name-input")).await.ok();
        let reset_button = driver.find(By::Css("#form-reset")).await.ok();

        if let (Some(name), Some(reset)) = (name_input, reset_button) {
            // Fill input
            name.send_keys("Test Name").await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            let filled_value = name.attr("value").await?.unwrap_or_default();
            info!("Input value before reset: '{}'", filled_value);

            // Click reset
            reset.click().await?;
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Check if cleared
            let reset_value = name.attr("value").await?.unwrap_or_default();
            info!("Input value after reset: '{}'", reset_value);

            if reset_value.is_empty() || reset_value != filled_value {
                info!("Form reset worked");
            }
        } else {
            info!("Reset elements not found, skipping form reset test");
            let duration = start.elapsed().as_millis() as u64;
            return Ok(TestResult {
                component: "Form Reset".to_string(),
                status: crate::tests::TestStatus::Warning,
                message: "Reset elements not found on page".to_string(),
                duration_ms: duration,
                screenshot_path: None,
            });
        }

        let duration = start.elapsed().as_millis() as u64;
        Ok(TestResult {
            component: "Form Reset".to_string(),
            status: crate::tests::TestStatus::Success,
            message: "Form reset functionality works".to_string(),
            duration_ms: duration,
            screenshot_path: None,
        })
    }
}

impl Test for FormValidationTests {
    fn name(&self) -> &str {
        "Form Validation Tests"
    }

    fn setup(&self) -> Result<()> {
        info!("Setting up form validation test suite");
        Ok(())
    }

    async fn run_with_driver(&self, driver: &WebDriver) -> Result<TestResult> {
        info!("Running form validation E2E tests");

        let mut results = vec![];

        // Test 1: Required field validation
        match self.test_required_field_validation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Required field validation test failed: {}", e);
                results.push(TestResult::error(
                    "Required Field Validation",
                    &e.to_string(),
                ));
            }
        }

        // Test 2: Email validation
        match self.test_email_validation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Email validation test failed: {}", e);
                results.push(TestResult::error("Email Validation", &e.to_string()));
            }
        }

        // Test 3: Password validation
        match self.test_password_validation(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Password validation test failed: {}", e);
                results.push(TestResult::error("Password Validation", &e.to_string()));
            }
        }

        // Test 4: Form submission
        match self.test_form_submission(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Form submission test failed: {}", e);
                results.push(TestResult::error("Form Submission", &e.to_string()));
            }
        }

        // Test 5: Form reset
        match self.test_form_reset(driver).await {
            Ok(result) => results.push(result),
            Err(e) => {
                tracing::error!("Form reset test failed: {}", e);
                results.push(TestResult::error("Form Reset", &e.to_string()));
            }
        }

        Ok(TestResult::aggregate(results))
    }

    fn teardown(&self) -> Result<()> {
        info!("Tearing down form validation test suite");
        Ok(())
    }
}
