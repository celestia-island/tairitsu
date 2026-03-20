//! Chromium browser downloader

use anyhow::{Context, Result, bail};
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::fs::File;
use tracing::info;

use super::cache::BrowserCache;
use super::platform::Platform;

/// Chrome for Testing version (stable)
pub const CHROME_VERSION: &str = "146.0.7680.153";

/// Download progress information
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub downloaded: u64,
    pub total: u64,
}

/// Browser downloader
pub struct BrowserDownloader {
    client: Client,
    cache: BrowserCache,
    mirror: Option<String>,
}

impl BrowserDownloader {
    /// Create a new downloader
    pub fn new(cache: BrowserCache, mirror: Option<String>) -> Self {
        let client = Client::builder()
            .user_agent("tairitsu-browser-test/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, cache, mirror }
    }

    /// Get the download URL for a specific version and platform
    fn get_download_url(&self, version: &str, platform: Platform) -> String {
        let base_url = self.mirror.as_ref().map(|m| m.as_str()).unwrap_or_else(|| {
            "https://storage.googleapis.com/chrome-for-testing-public"
        });

        let platform_path = platform.download_id();
        format!(
            "{}/{}/{}/chrome-{}.zip",
            base_url, version, platform_path, platform_path
        )
    }

    /// Download and install Chromium
    pub async fn download(&self, version: &str, platform: Platform) -> Result<PathBuf> {
        let url = self.get_download_url(version, platform);
        info!("Downloading Chromium {} for {} from {}", version, platform, url);

        // Check if already cached
        let exec_path = self.cache.executable_path(version, platform);
        if exec_path.exists() {
            info!("Chromium already cached at {}", exec_path.display());
            return Ok(exec_path);
        }

        // Start download
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to start download")?;

        if !response.status().is_success() {
            bail!("Download failed with status: {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let progress = ProgressBar::new(total_size)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                    .unwrap()
                    .progress_chars("##-")
            );

        // Download to temporary file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join(format!("chrome-{}.zip", version));

        let mut file = File::create(&temp_file)
            .context("Failed to create temporary file")?;

        let mut _hasher = Sha256::new();
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Download stream error")?;
            _hasher.update(&chunk);
            file.write_all(&chunk)
                .context("Failed to write downloaded data")?;
            progress.inc(chunk.len() as u64);
        }

        progress.finish();
        drop(file);

        // Extract archive
        info!("Extracting Chromium archive...");
        let target_dir = self.cache.browser_path(version, platform);
        self.extract_archive(&temp_file, &target_dir, platform)?;

        // Clean up temp file
        std::fs::remove_file(&temp_file)
            .context("Failed to remove temporary file")?;

        let exec_path = self.cache.executable_path(version, platform);
        info!("Chromium installed at {}", exec_path.display());

        Ok(exec_path)
    }

    /// Extract the downloaded archive
    fn extract_archive(&self, archive_path: &Path, target_dir: &Path, platform: Platform) -> Result<()> {
        std::fs::create_dir_all(target_dir)
            .context("Failed to create target directory")?;

        let file = File::open(archive_path)
            .context("Failed to open archive")?;

        let mut archive = zip::ZipArchive::new(std::io::BufReader::new(file))
            .context("Failed to read ZIP archive")?;

        let progress = ProgressBar::new(archive.len() as u64)
            .with_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {pos}/{len} files")
                    .unwrap()
            );

        let exec_name = platform.executable_name();

        for i in 0..archive.len() {
            let mut zip_file = archive.by_index(i)
                .context("Failed to get ZIP file entry")?;

            let mangled_name = zip_file.mangled_name();
            let out_path = target_dir.join(&mangled_name);

            if zip_file.name().ends_with('/') {
                std::fs::create_dir_all(&out_path)
                    .context("Failed to create directory")?;
            } else {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)
                        .context("Failed to create parent directory")?;
                }
                let mut out_file = File::create(&out_path)
                    .context("Failed to create output file")?;
                std::io::copy(&mut zip_file, &mut out_file)
                    .context("Failed to extract file")?;

                // Set executable permission on binaries and shared libraries
                let needs_exec = mangled_name.file_name().map(|n| {
                    n == exec_name ||
                    n == "chrome_crashpad_handler" ||
                    n.to_string_lossy().ends_with(".so")
                }).unwrap_or(false);

                if needs_exec {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(0o755))
                            .context("Failed to set executable permissions")?;
                    }
                }
            }
            progress.inc(1);
        }

        progress.finish();
        Ok(())
    }
}
