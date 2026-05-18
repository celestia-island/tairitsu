//! Browser cache management

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::info;

use super::platform::Platform;

/// Default cache directory name
const CACHE_DIR_NAME: &str = "browsers";

/// Browser cache manager
pub struct BrowserCache {
    cache_dir: PathBuf,
}

impl BrowserCache {
    /// Create a new cache manager
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| {
            dirs::cache_dir()
                .expect("Failed to get cache directory")
                .join("tairitsu")
                .join(CACHE_DIR_NAME)
        });
        Self { cache_dir }
    }

    /// Get the cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get the path to a cached browser version directory
    pub fn browser_path(&self, version: &str, platform: Platform) -> PathBuf {
        self.cache_dir
            .join("chromium")
            .join(version)
            .join(platform.download_id())
    }

    /// Check if a browser version is cached
    pub fn is_cached(&self, version: &str, platform: Platform) -> bool {
        let exec_path = self.executable_path(version, platform);
        exec_path.exists()
    }

    /// Get the executable path for a cached browser
    pub fn executable_path(&self, version: &str, platform: Platform) -> PathBuf {
        // Chrome for Testing archives extract with a directory structure like:
        // chrome-linux64/chrome
        // chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing
        // chrome-win64/chrome.exe
        let download_id = platform.download_id();
        self.browser_path(version, platform)
            .join(format!("chrome-{}", download_id))
            .join(platform.executable_relative_path())
    }

    /// List all cached browser versions
    pub fn list_cached(&self) -> Result<Vec<(String, Platform)>> {
        let mut cached = Vec::new();
        let chromium_dir = self.cache_dir.join("chromium");

        if !chromium_dir.exists() {
            return Ok(cached);
        }

        let entries =
            fs::read_dir(&chromium_dir).context("Failed to read chromium cache directory")?;

        for entry in entries {
            let entry = entry.context("Reading cache directory entry")?;
            let file_type = entry.file_type().context("Reading file type")?;
            if file_type.is_dir() {
                let version = entry.file_name();
                let version_str = version.to_string_lossy().into_owned();

                // Check for cached platforms within this version
                let version_dir = entry.path();
                let platform_entries = match fs::read_dir(&version_dir) {
                    Ok(entries) => entries,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to read version directory {}: {}",
                            version_dir.display(),
                            e
                        );
                        continue;
                    }
                };

                for platform_entry in platform_entries {
                    let platform_entry =
                        platform_entry.context("Reading version directory entry")?;
                    let file_type = platform_entry.file_type().context("Reading file type")?;
                    if file_type.is_dir() {
                        let platform_name = platform_entry.file_name();
                        let platform_str = platform_name.to_string_lossy();

                        // Parse platform and check if executable exists
                        if let Some(platform) = parse_platform(&platform_str) {
                            let exec_path = self.executable_path(&version_str, platform);
                            if exec_path.exists() {
                                cached.push((version_str.clone(), platform));
                            }
                        }
                    }
                }
            }
        }

        Ok(cached)
    }

    /// Clear the entire browser cache
    pub fn clear(&self) -> Result<()> {
        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir).context("Clearing browser cache")?;
            info!("Cleared browser cache: {}", self.cache_dir.display());
        }
        Ok(())
    }

    /// Ensure the cache directory exists
    pub fn ensure_cache_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.cache_dir).context("Creating cache directory")?;
        Ok(())
    }
}

fn parse_platform(s: &str) -> Option<Platform> {
    match s {
        "linux-x64" => Some(Platform::LinuxX64),
        "mac-arm64" => Some(Platform::MacosArm64),
        "mac-x64" => Some(Platform::MacosX64),
        "win-x64" => Some(Platform::WindowsX64),
        _ => None,
    }
}
