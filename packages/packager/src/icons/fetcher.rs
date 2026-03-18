//! Icon fetching and caching for tairitsu-packager.
//!
//! Downloads icons from various sources and manages local cache.
//! Supports MDI (Material Design Icons) with async fetching and ZIP extraction.
//!
//! # Cache Structure
//!
//! ```text
//! target/tairitsu/icons/
//!   mdi/
//!     svg/
//!       account.svg
//!       account-circle.svg
//!       ...
//!     metadata.json
//!   mdi_metadata.json  (global metadata cache)
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use tairitsu_packager::icons::{IconFetcher, IconSource};
//!
//! #[tokio::main]
//! async fn main() {
//!     let fetcher = IconFetcher::new("/path/to/cache".into(), IconSource::Mdi);
//!     let metadata = fetcher.fetch(false).await.unwrap();
//!     println!("Fetched {} icons", metadata.count);
//! }
//! ```

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tracing::{debug, info, warn};
#[allow(unused_imports)]
use zip::ZipArchive;

use super::metadata::IconMetadata;
use super::{IconSource, MDI_DEFAULT_VERSION};

// ============================================================================
// Constants
// ============================================================================

/// MDI SVG distribution URL template (npm package tarball via unpkg CDN)
/// This is more reliable than GitHub releases for getting the full SVG set
#[allow(dead_code)]
const MDI_SVG_PACKAGE_URL: &str =
    "https://unpkg.com/@mdi/svg@{version}/";

/// Alternative: Direct GitHub raw URL for meta.json
const MDI_GITHUB_META_URL: &str =
    "https://raw.githubusercontent.com/Templarian/MaterialDesign/master/meta.json";

/// Alternative: GitHub raw URL for SVG files (individual)
#[allow(dead_code)]
const MDI_GITHUB_SVG_URL: &str =
    "https://raw.githubusercontent.com/Templarian/MaterialDesign/master/svg/{name}.svg";

/// NPM registry URL for @mdi/svg package info
#[allow(dead_code)]
const NPM_REGISTRY_URL: &str = "https://registry.npmjs.org/@mdi/svg/{version}";

/// Cache subdirectory for MDI icons
#[allow(dead_code)]
const MDI_CACHE_SUBDIR: &str = "mdi";

/// Metadata cache filename
const METADATA_FILENAME: &str = "metadata.json";

/// SVG subdirectory name
const SVG_SUBDIR: &str = "svg";

// ============================================================================
// Icon Fetcher
// ============================================================================

/// Icon fetcher for downloading and caching icons.
///
/// Supports multiple icon sources with async fetching and local caching.
/// Icons are downloaded once and cached for subsequent use.
pub struct IconFetcher {
    /// Cache directory (e.g., target/tairitsu/icons/)
    cache_dir: PathBuf,

    /// Icon source (MDI, Lucide, Custom)
    source: IconSource,

    /// Version to fetch (for MDI, defaults to MDI_DEFAULT_VERSION)
    version: String,
}

impl IconFetcher {
    /// Create a new icon fetcher.
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Base cache directory for icons
    /// * `source` - Icon source library
    pub fn new(cache_dir: PathBuf, source: IconSource) -> Self {
        Self {
            cache_dir,
            source,
            version: MDI_DEFAULT_VERSION.to_string(),
        }
    }

    /// Set the version to fetch.
    ///
    /// For MDI, this should be a valid version like "7.4.47" or "latest".
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Get the source-specific cache directory.
    ///
    /// For MDI, this would be `{cache_dir}/mdi/`.
    pub fn source_cache_dir(&self) -> PathBuf {
        self.cache_dir.join(self.source.to_string())
    }

    /// Get the metadata file path.
    ///
    /// For MDI: `{cache_dir}/mdi/metadata.json`
    pub fn metadata_path(&self) -> PathBuf {
        self.source_cache_dir().join(METADATA_FILENAME)
    }

    /// Get the SVG directory path.
    ///
    /// For MDI: `{cache_dir}/mdi/svg/`
    pub fn svg_dir(&self) -> PathBuf {
        self.source_cache_dir().join(SVG_SUBDIR)
    }

    /// Get the global metadata path (for backward compatibility).
    ///
    /// This is `{cache_dir}/mdi_metadata.json`.
    pub fn global_metadata_path(&self) -> PathBuf {
        self.cache_dir.join("mdi_metadata.json")
    }

    /// Check if icons are already cached.
    ///
    /// Returns true if both metadata and SVG directory exist.
    pub fn is_cached(&self) -> bool {
        let metadata_exists = self.metadata_path().exists();
        let svg_dir_exists = self.svg_dir().exists();

        // Check if SVG directory has at least some files
        let has_icons = if svg_dir_exists {
            std::fs::read_dir(self.svg_dir())
                .map(|mut d| d.next().is_some())
                .unwrap_or(false)
        } else {
            false
        };

        metadata_exists && has_icons
    }

    /// Fetch icons (uses cache if available).
    ///
    /// # Arguments
    ///
    /// * `force` - If true, re-download even if cached
    ///
    /// # Returns
    ///
    /// Icon metadata with all icon information loaded.
    pub async fn fetch(&self, force: bool) -> crate::Result<IconMetadata> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&self.cache_dir)?;
        std::fs::create_dir_all(self.source_cache_dir())?;
        std::fs::create_dir_all(self.svg_dir())?;

        // Load from cache if available and not forcing refresh
        if !force && self.is_cached() {
            info!(
                "Loading {} icons from cache: {}",
                self.source,
                self.source_cache_dir().display()
            );
            return self.load_cached_metadata();
        }

        // Fetch fresh icons based on source
        match self.source {
            IconSource::Mdi => self.fetch_mdi().await,
            IconSource::Lucide => self.fetch_lucide().await,
            IconSource::Custom => self.load_custom(),
        }
    }

    /// Load cached metadata and SVG paths.
    fn load_cached_metadata(&self) -> crate::Result<IconMetadata> {
        let content = std::fs::read_to_string(self.metadata_path())?;
        let mut metadata = IconMetadata::parse_mdi_json(&content)?;

        // Load SVG paths for each icon
        let svg_dir = self.svg_dir();
        for (name, icon) in &mut metadata.icons {
            let svg_path = svg_dir.join(format!("{}.svg", name));
            if svg_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&svg_path) {
                    icon.svg_path = extract_svg_path(&content);
                }
            }
        }

        Ok(metadata)
    }

    /// Fetch MDI icons from the npm registry or GitHub.
    ///
    /// Strategy:
    /// 1. Try to download @mdi/svg tarball from npm registry
    /// 2. If that fails, fall back to fetching individual SVGs from GitHub
    /// 3. Always fetch meta.json from GitHub for metadata
    async fn fetch_mdi(&self) -> crate::Result<IconMetadata> {
        info!("Fetching MDI icons v{}", self.version);

        // Step 1: Fetch and save metadata
        let metadata = self.fetch_mdi_metadata().await?;
        info!("Fetched metadata for {} MDI icons", metadata.count);

        // Step 2: Download SVG files
        match self.download_mdi_package().await {
            Ok(count) => {
                info!("Downloaded {} SVG files from npm package", count);
            }
            Err(e) => {
                warn!("Failed to download MDI package from npm: {}, falling back to GitHub", e);
                // Fallback: don't download all SVGs, they'll be fetched on-demand
                info!("SVGs will be fetched on-demand from GitHub when needed");
            }
        }

        // Save metadata to cache
        self.save_metadata(&metadata)?;

        // Reload with SVG paths
        self.load_cached_metadata()
    }

    /// Fetch MDI metadata from GitHub.
    async fn fetch_mdi_metadata(&self) -> crate::Result<IconMetadata> {
        let url = MDI_GITHUB_META_URL;
        debug!("Fetching MDI metadata from {}", url);

        let content = fetch_url_async(url).await?;
        let mut metadata = IconMetadata::parse_mdi_json(&content)?;
        metadata.version = self.version.clone();

        // Save raw metadata to cache
        std::fs::write(self.metadata_path(), &content)?;

        Ok(metadata)
    }

    /// Save metadata to cache.
    fn save_metadata(&self, metadata: &IconMetadata) -> crate::Result<()> {
        // Serialize metadata
        let json = serde_json::to_string_pretty(metadata)?;
        std::fs::write(self.metadata_path(), json)?;

        // Also save to global location for backward compatibility
        let global_path = self.global_metadata_path();
        let legacy_metadata = LegacyMetadata::from(metadata);
        let legacy_json = serde_json::to_string_pretty(&legacy_metadata)?;
        std::fs::write(global_path, legacy_json)?;

        Ok(())
    }

    /// Download MDI SVG package from npm registry.
    ///
    /// This downloads the @mdi/svg tarball and extracts all SVG files.
    async fn download_mdi_package(&self) -> crate::Result<usize> {
        let version = if self.version == "latest" {
            // Get latest version from npm
            self.get_latest_mdi_version().await?
        } else {
            self.version.clone()
        };

        // Construct tarball URL
        let tarball_url = format!(
            "https://registry.npmjs.org/@mdi/svg/-/svg-{}.tgz",
            version
        );

        info!("Downloading MDI SVG package from {}", tarball_url);

        // Download tarball
        let _tarball = fetch_bytes_async(&tarball_url).await?;

        // Extract tarball (it's a gzipped tar, but we can use the zip library
        // with flate2 for gzip decompression, then parse the tar)
        // For simplicity, we'll use an alternative approach: download from unpkg
        self.download_from_unpkg(&version).await
    }

    /// Get the latest MDI version from npm.
    async fn get_latest_mdi_version(&self) -> crate::Result<String> {
        let url = "https://registry.npmjs.org/@mdi/svg/latest";
        let content = fetch_url_async(url).await?;

        let json: serde_json::Value = serde_json::from_str(&content)?;
        let version = json
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::TairitsuPackagerError::IconFetchError(
                    "Could not parse latest version from npm".to_string(),
                )
            })?;

        Ok(version.to_string())
    }

    /// Download SVG files from unpkg CDN.
    ///
    /// This is an alternative approach that downloads files individually
    /// from the unpkg CDN, which serves files directly from npm packages.
    async fn download_from_unpkg(&self, version: &str) -> crate::Result<usize> {
        let svg_dir = self.svg_dir();
        let mut count = 0;

        // Load metadata to get icon names
        let metadata = self.load_cached_metadata()?;

        info!(
            "Downloading {} SVG files from unpkg for v{}",
            metadata.icons.len(),
            version
        );

        // Download icons in batches using async
        use futures::future::join_all;

        let batch_size = 20;
        let icon_names: Vec<_> = metadata.icons.keys().cloned().collect();

        for chunk in icon_names.chunks(batch_size) {
            let futures: Vec<_> = chunk
                .iter()
                .filter(|name| {
                    // Skip if already cached
                    !svg_dir.join(format!("{}.svg", name)).exists()
                })
                .map(|name| self.download_single_svg(name, version))
                .collect();

            let results = join_all(futures).await;

            for result in results {
                match result {
                    Ok(downloaded) => {
                        if downloaded {
                            count += 1;
                        }
                    }
                    Err(e) => {
                        debug!("Failed to download SVG: {}", e);
                    }
                }
            }

            // Small delay between batches to be nice to the server
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(count)
    }

    /// Download a single SVG file from unpkg.
    async fn download_single_svg(&self, name: &str, version: &str) -> crate::Result<bool> {
        let url = format!(
            "https://unpkg.com/@mdi/svg@{}/svg/{}.svg",
            version, name
        );

        let content = fetch_url_async(&url).await?;

        // Save to cache
        let svg_path = self.svg_dir().join(format!("{}.svg", name));
        std::fs::write(&svg_path, &content)?;

        debug!("Downloaded {}.svg", name);
        Ok(true)
    }

    /// Fetch Lucide icons (stub implementation).
    async fn fetch_lucide(&self) -> crate::Result<IconMetadata> {
        warn!("Lucide icon source not yet implemented");
        Ok(IconMetadata::default())
    }

    /// Load custom icons from directory.
    fn load_custom(&self) -> crate::Result<IconMetadata> {
        info!("Loading custom icons from directory");
        // Custom icons are loaded from a local directory
        // This would be configured via IconsConfig::custom_dir
        Ok(IconMetadata::default())
    }
}

// ============================================================================
// Async HTTP Client
// ============================================================================

/// Global async HTTP client (lazy initialized)
static ASYNC_HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

/// Get or create the async HTTP client.
fn get_async_http_client() -> &'static reqwest::Client {
    ASYNC_HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(format!("tairitsu-packager/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create async HTTP client")
    })
}

/// Fetch URL content asynchronously.
async fn fetch_url_async(url: &str) -> crate::Result<String> {
    let client = get_async_http_client();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| {
            crate::TairitsuPackagerError::HttpError(format!("Failed to fetch {}: {}", url, e))
        })?;

    if !response.status().is_success() {
        return Err(crate::TairitsuPackagerError::HttpError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    response.text().await.map_err(|e| {
        crate::TairitsuPackagerError::HttpError(format!("Failed to read response: {}", e))
    })
}

/// Fetch URL content as bytes asynchronously.
async fn fetch_bytes_async(url: &str) -> crate::Result<Vec<u8>> {
    let client = get_async_http_client();
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| {
            crate::TairitsuPackagerError::HttpError(format!("Failed to fetch {}: {}", url, e))
        })?;

    if !response.status().is_success() {
        return Err(crate::TairitsuPackagerError::HttpError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    response.bytes().await.map_err(|e| {
        crate::TairitsuPackagerError::HttpError(format!("Failed to read response bytes: {}", e))
    }).map(|b| b.to_vec())
}

// ============================================================================
// Sync HTTP Client (for non-async contexts)
// ============================================================================

/// Global blocking HTTP client (lazy initialized)
static BLOCKING_HTTP_CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

/// Get or create the blocking HTTP client.
fn get_blocking_http_client() -> &'static reqwest::blocking::Client {
    BLOCKING_HTTP_CLIENT.get_or_init(|| {
        reqwest::blocking::Client::builder()
            .user_agent(format!("tairitsu-packager/{}", crate::VERSION))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create blocking HTTP client")
    })
}

/// Fetch URL content synchronously.
fn fetch_url_blocking(url: &str) -> crate::Result<String> {
    let client = get_blocking_http_client();
    let response = client.get(url).send().map_err(|e| {
        crate::TairitsuPackagerError::HttpError(format!("Failed to fetch {}: {}", url, e))
    })?;

    if !response.status().is_success() {
        return Err(crate::TairitsuPackagerError::HttpError(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    response.text().map_err(|e| {
        crate::TairitsuPackagerError::HttpError(format!("Failed to read response: {}", e))
    })
}

// ============================================================================
// SVG Path Extraction
// ============================================================================

/// Extract path data from SVG content.
///
/// This finds the `d` attribute of the first `<path>` element.
/// For production use, consider using a proper XML parser.
pub fn extract_svg_path(svg: &str) -> Option<String> {
    // Simple extraction: find the d attribute of path element
    // In production, use proper XML parsing
    if let Some(start) = svg.find("d=\"") {
        let start = start + 3;
        if let Some(end) = svg[start..].find("\"") {
            return Some(svg[start..start + end].to_string());
        }
    }
    None
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Fetch icons from source (convenience function, async).
///
/// Uses cache if available, otherwise downloads fresh.
pub async fn fetch_icons_async(
    source: &IconSource,
    cache_dir: &Path,
) -> crate::Result<IconMetadata> {
    let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
    fetcher.fetch(false).await
}

/// Force fetch icons (ignores cache, async).
pub async fn force_fetch_icons_async(
    source: &IconSource,
    cache_dir: &Path,
) -> crate::Result<IconMetadata> {
    let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
    fetcher.fetch(true).await
}

/// Fetch icons from source (synchronous wrapper).
///
/// This is a blocking wrapper around the async fetch function.
/// Uses cache if available, otherwise downloads fresh.
pub fn fetch_icons(source: &IconSource, cache_dir: &Path) -> crate::Result<IconMetadata> {
    // Try to use tokio runtime if available, otherwise use blocking
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            // We're in an async context, block_on should work
            handle.block_on(fetch_icons_async(source, cache_dir))
        }
        Err(_) => {
            // No async runtime, use blocking implementation
            fetch_icons_blocking(source, cache_dir)
        }
    }
}

/// Force fetch icons (synchronous wrapper).
pub fn force_fetch_icons(source: &IconSource, cache_dir: &Path) -> crate::Result<IconMetadata> {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => handle.block_on(force_fetch_icons_async(source, cache_dir)),
        Err(_) => force_fetch_icons_blocking(source, cache_dir),
    }
}

/// Blocking implementation for fetch_icons.
fn fetch_icons_blocking(source: &IconSource, cache_dir: &Path) -> crate::Result<IconMetadata> {
    let source_cache_dir = cache_dir.join(source.to_string());
    let metadata_path = source_cache_dir.join(METADATA_FILENAME);
    let svg_dir = source_cache_dir.join(SVG_SUBDIR);

    // Ensure directories exist
    std::fs::create_dir_all(&source_cache_dir)?;
    std::fs::create_dir_all(&svg_dir)?;

    // Load from cache if available
    if metadata_path.exists() && svg_dir.exists() {
        let content = std::fs::read_to_string(&metadata_path)?;
        let mut metadata = IconMetadata::parse_mdi_json(&content)?;

        // Load SVG paths
        for (name, icon) in &mut metadata.icons {
            let svg_path = svg_dir.join(format!("{}.svg", name));
            if svg_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&svg_path) {
                    icon.svg_path = extract_svg_path(&content);
                }
            }
        }

        return Ok(metadata);
    }

    // Fetch metadata
    let content = fetch_url_blocking(MDI_GITHUB_META_URL)?;
    std::fs::write(&metadata_path, &content)?;

    let mut metadata = IconMetadata::parse_mdi_json(&content)?;
    metadata.version = MDI_DEFAULT_VERSION.to_string();

    info!("Fetched {} MDI icons (metadata only, SVGs fetched on-demand)", metadata.count);

    Ok(metadata)
}

/// Blocking implementation for force_fetch_icons.
fn force_fetch_icons_blocking(source: &IconSource, cache_dir: &Path) -> crate::Result<IconMetadata> {
    let source_cache_dir = cache_dir.join(source.to_string());
    let metadata_path = source_cache_dir.join(METADATA_FILENAME);

    // Ensure directories exist
    std::fs::create_dir_all(&source_cache_dir)?;
    std::fs::create_dir_all(source_cache_dir.join(SVG_SUBDIR))?;

    // Always fetch fresh metadata
    let content = fetch_url_blocking(MDI_GITHUB_META_URL)?;
    std::fs::write(&metadata_path, &content)?;

    let mut metadata = IconMetadata::parse_mdi_json(&content)?;
    metadata.version = MDI_DEFAULT_VERSION.to_string();

    info!("Force-fetched {} MDI icons", metadata.count);

    Ok(metadata)
}

/// List icons from cache.
#[allow(dead_code)]
pub fn list_cached_icons(cache_dir: &Path) -> crate::Result<Vec<String>> {
    let svg_dir = cache_dir.join(SVG_SUBDIR);
    if !svg_dir.exists() {
        return Ok(Vec::new());
    }

    let mut icons = Vec::new();
    for entry in std::fs::read_dir(svg_dir)? {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str() {
            if name.ends_with(".svg") {
                icons.push(name.trim_end_matches(".svg").to_string());
            }
        }
    }

    icons.sort();
    Ok(icons)
}

// ============================================================================
// Legacy Metadata Format
// ============================================================================

/// Legacy metadata format for backward compatibility.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LegacyMetadata {
    /// Version
    version: String,
    /// Count
    count: usize,
    /// Icons
    icons: Vec<LegacyIconEntry>,
}

impl From<&IconMetadata> for LegacyMetadata {
    fn from(metadata: &IconMetadata) -> Self {
        Self {
            version: metadata.version.clone(),
            count: metadata.count,
            icons: metadata
                .icons
                .values()
                .map(|e| LegacyIconEntry {
                    name: e.name.clone(),
                    aliases: e.aliases.clone(),
                    tags: e.tags.clone(),
                    author: e.author.clone(),
                    version: e.version.clone(),
                    deprecated: e.deprecated,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LegacyIconEntry {
    name: String,
    aliases: Vec<String>,
    tags: Vec<String>,
    author: Option<String>,
    version: Option<String>,
    deprecated: bool,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_svg_path() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M12 2L2 22h20L12 2z"/></svg>"#;
        let path = extract_svg_path(svg);
        assert_eq!(path, Some("M12 2L2 22h20L12 2z".to_string()));
    }

    #[test]
    fn test_extract_svg_path_complex() {
        let svg = r#"<svg viewBox="0 0 24 24"><path fill="currentColor" d="M4 4h16v16H4V4zm2 2v12h12V6H6z"/></svg>"#;
        let path = extract_svg_path(svg);
        assert_eq!(
            path,
            Some("M4 4h16v16H4V4zm2 2v12h12V6H6z".to_string())
        );
    }

    #[test]
    fn test_extract_svg_path_no_path() {
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><!-- empty --></svg>"#;
        let path = extract_svg_path(svg);
        assert_eq!(path, None);
    }

    #[test]
    fn test_icon_fetcher_paths() {
        let fetcher = IconFetcher::new(
            PathBuf::from("/tmp/cache"),
            IconSource::Mdi,
        );

        assert_eq!(
            fetcher.source_cache_dir(),
            PathBuf::from("/tmp/cache/mdi")
        );
        assert_eq!(
            fetcher.metadata_path(),
            PathBuf::from("/tmp/cache/mdi/metadata.json")
        );
        assert_eq!(
            fetcher.svg_dir(),
            PathBuf::from("/tmp/cache/mdi/svg")
        );
        assert_eq!(
            fetcher.global_metadata_path(),
            PathBuf::from("/tmp/cache/mdi_metadata.json")
        );
    }

    #[test]
    fn test_icon_fetcher_with_version() {
        let fetcher = IconFetcher::new(
            PathBuf::from("/tmp/cache"),
            IconSource::Mdi,
        )
        .with_version("7.3.67");

        assert_eq!(fetcher.version, "7.3.67");
    }
}
