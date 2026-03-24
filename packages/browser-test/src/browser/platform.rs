//! Platform detection for browser downloads

use std::env;
use std::fmt;

/// Supported platforms for browser downloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    MacosArm64,
    MacosX64,
    WindowsX64,
}

impl Platform {
    /// Get the platform identifier used in download URLs
    pub fn download_id(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "linux64",
            Platform::MacosArm64 => "mac-arm64",
            Platform::MacosX64 => "mac-x64",
            Platform::WindowsX64 => "win64",
        }
    }

    /// Get the executable name for this platform
    pub fn executable_name(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "chrome",
            Platform::MacosArm64 => "Google Chrome for Testing",
            Platform::MacosX64 => "Google Chrome for Testing",
            Platform::WindowsX64 => "chrome.exe",
        }
    }

    /// Get the relative path to the executable within the extracted archive
    pub fn executable_relative_path(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "chrome",
            Platform::MacosArm64 => {
                "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
            }
            Platform::MacosX64 => {
                "Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing"
            }
            Platform::WindowsX64 => "chrome.exe",
        }
    }

    /// Get the archive file extension for this platform
    pub fn archive_extension(&self) -> &'static str {
        match self {
            Platform::LinuxX64 => "zip",
            Platform::MacosArm64 => "zip",
            Platform::MacosX64 => "zip",
            Platform::WindowsX64 => "zip",
        }
    }

    /// Check if this platform is macOS
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacosArm64 | Platform::MacosX64)
    }

    /// Check if this platform is Windows
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::WindowsX64)
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::LinuxX64 => write!(f, "linux64"),
            Platform::MacosArm64 => write!(f, "mac-arm64"),
            Platform::MacosX64 => write!(f, "mac-x64"),
            Platform::WindowsX64 => write!(f, "win64"),
        }
    }
}

/// Detect the current platform
pub fn detect_platform() -> Platform {
    match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86_64") => Platform::LinuxX64,
        ("macos", "aarch64") => Platform::MacosArm64,
        ("macos", "x86_64") => Platform::MacosX64,
        ("windows", "x86_64") => Platform::WindowsX64,
        ("windows", "x86") => Platform::WindowsX64,
        _ => panic!(
            "Unsupported platform: {}-{}",
            env::consts::OS,
            env::consts::ARCH
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_download_id() {
        assert_eq!(Platform::LinuxX64.download_id(), "linux64");
        assert_eq!(Platform::MacosArm64.download_id(), "mac-arm64");
    }

    #[test]
    fn test_platform_executable_name() {
        assert_eq!(Platform::LinuxX64.executable_name(), "chrome");
        assert_eq!(Platform::WindowsX64.executable_name(), "chrome.exe");
    }
}
