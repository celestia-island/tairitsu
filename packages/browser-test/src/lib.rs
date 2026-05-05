//! Browser testing infrastructure for Tairitsu
//!
//! This crate provides:
//! - Automatic Chromium browser downloading and caching
//! - Browser automation for testing browser-glue functionality
//! - CI-friendly test execution with mirror/proxy support

pub mod browser;
pub mod cli;
pub mod runner;

pub use browser::{BrowserCache, BrowserDownloader};
pub use runner::{TestHarness, TestResult};
