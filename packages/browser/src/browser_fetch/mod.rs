//! Simplified Chrome resolver — finds Chrome on the system.
//! For auto-download, see the `auto-fetch` feature (TODO).

use std::path::PathBuf;

/// Resolve Chrome/Chromium executable path.
///
/// Order:
/// 1. $CHROME_PATH env var
/// 2. Common system paths
/// 3. Playwright cache
pub fn resolve_executable() -> Result<String, String> {
    // 1. Explicit override
    if let Ok(path) = std::env::var("CHROME_PATH") {
        if !path.is_empty() {
            return Ok(path);
        }
    }

    // 2. System PATH
    let candidates = [
        "chromium-browser",
        "chromium",
        "google-chrome",
        "google-chrome-stable",
        "chrome",
    ];
    for name in &candidates {
        if let Ok(output) = std::process::Command::new("which").arg(name).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }
    }

    // 3. Playwright cache
    let pw_paths = [
        dirs_home().join(".cache/ms-playwright"),
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".cache/ms-playwright"),
    ];
    for pw_dir in &pw_paths {
        if pw_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(pw_dir) {
                for entry in entries.flatten() {
                    let chromium_dir = entry.path().join("chrome-linux64");
                    let chrome = chromium_dir.join("chrome");
                    if chrome.exists() {
                        return Ok(chrome.to_string_lossy().to_string());
                    }
                    // Also check chrome-linux (32-bit naming)
                    let chromium32 = entry.path().join("chrome-linux");
                    let chrome32 = chromium32.join("chrome");
                    if chrome32.exists() {
                        return Ok(chrome32.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Err("Chrome/Chromium not found. Set CHROME_PATH or install chromium-browser.".to_string())
}

fn dirs_home() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
}
