//! Icon fetching and caching for tairitsu-packager.
//!
//! This module provides icon loading capabilities. HTTP fetching is optional
//! and can be enabled via the `icon-fetch` feature flag.
//!
//! # Simple File-Based Usage (Default)
//!
//! ```rust,ignore
//! use tairitsu_packager::icons::{IconFetcher, IconSource};
//!
//! // Load icons from local directory
//! let fetcher = IconFetcher::new("/path/to/icons".into(), IconSource::Custom);
//! let metadata = fetcher.load_from_directory()?;
//! ```
//!
//! # HTTP Fetching (Requires `icon-fetch` feature)
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

use tracing::{info, warn};

use super::metadata::IconMetadata;
use super::{IconSource, MDI_DEFAULT_VERSION};

// ============================================================================
// Constants
// ============================================================================

/// Metadata cache filename
const METADATA_FILENAME: &str = "metadata.json";

/// SVG subdirectory name
const SVG_SUBDIR: &str = "svg";

// ============================================================================
// Icon Fetcher
// ============================================================================

/// Icon fetcher for loading icons.
///
/// Supports local file-based loading (always available) and optional
/// HTTP fetching (requires `icon-fetch` feature).
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

    /// Load icons from a local directory (simple file-based approach).
    ///
    /// This scans the `icons/` directory for SVG files and builds metadata.
    /// This is the recommended simple approach.
    pub fn load_from_directory(&self) -> crate::Result<IconMetadata> {
        let icons_dir = self.cache_dir.join("icons");

        if !icons_dir.exists() {
            return Ok(IconMetadata::default());
        }

        let mut metadata = IconMetadata {
            version: "local".to_string(),
            ..Default::default()
        };

        for entry in std::fs::read_dir(&icons_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "svg").unwrap_or(false) {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = std::fs::read_to_string(&path)?;
                    let svg_path = extract_svg_path(&content);

                    metadata.icons.insert(
                        name.to_string(),
                        crate::icons::IconEntry {
                            name: name.to_string(),
                            aliases: vec![],
                            tags: vec![],
                            author: None,
                            version: None,
                            deprecated: false,
                            svg_path,
                        },
                    );
                }
            }
        }

        metadata.count = metadata.icons.len();
        info!(
            "Loaded {} icons from {}",
            metadata.count,
            icons_dir.display()
        );

        Ok(metadata)
    }

    /// Load cached metadata (no HTTP fetching).
    pub fn load_cached(&self) -> crate::Result<IconMetadata> {
        let metadata_path = self.metadata_path();
        let svg_dir = self.svg_dir();

        if !metadata_path.exists() {
            return Ok(IconMetadata::default());
        }

        let content = std::fs::read_to_string(&metadata_path)?;
        let mut metadata = IconMetadata::parse_mdi_json(&content)?;

        // Load SVG paths for each icon
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

    #[cfg(feature = "icon-fetch")]
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
            return self.load_cached();
        }

        // Fetch fresh icons based on source
        match self.source {
            IconSource::Mdi => self.fetch_mdi().await,
            IconSource::Lucide => self.fetch_lucide().await,
            IconSource::Custom => self.load_from_directory(),
        }
    }

    #[cfg(not(feature = "icon-fetch"))]
    /// Fetch icons - returns error when HTTP fetching is not enabled.
    ///
    /// Enable the `icon-fetch` feature to use HTTP fetching.
    pub async fn fetch(&self, _force: bool) -> crate::Result<IconMetadata> {
        match self.source {
            IconSource::Custom => self.load_from_directory(),
            _ => {
                warn!(
                    "HTTP icon fetching not available (compile without 'icon-fetch' feature). \
                     Use IconSource::Custom for local files."
                );
                self.load_cached()
            }
        }
    }
}

// ============================================================================
// HTTP Fetching (Optional)
// ============================================================================

#[cfg(feature = "icon-fetch")]
mod http_fetch {
    use super::*;
    use std::sync::OnceLock;

    /// MDI GitHub raw URL for meta.json
    const MDI_GITHUB_META_URL: &str =
        "https://raw.githubusercontent.com/Templarian/MaterialDesign/master/meta.json";

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
    pub async fn fetch_url_async(url: &str) -> crate::Result<String> {
        let client = get_async_http_client();
        let response = client.get(url).send().await.map_err(|e| {
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

    impl IconFetcher {
        /// Fetch MDI icons from GitHub.
        pub(super) async fn fetch_mdi(&self) -> crate::Result<IconMetadata> {
            info!("Fetching MDI icons v{}", self.version);

            // Fetch metadata from GitHub
            let content = fetch_url_async(MDI_GITHUB_META_URL).await?;
            let mut metadata = IconMetadata::parse_mdi_json(&content)?;
            metadata.version = self.version.clone();

            // Save metadata to cache
            std::fs::write(self.metadata_path(), &content)?;

            info!("Fetched metadata for {} MDI icons", metadata.count);

            // Load SVG paths from cache
            self.load_cached()
        }

        /// Fetch Lucene icons (stub implementation).
        pub(super) async fn fetch_lucide(&self) -> crate::Result<IconMetadata> {
            warn!("Lucide icon source not yet implemented");
            Ok(IconMetadata::default())
        }
    }
}

#[cfg(feature = "icon-fetch")]
use http_fetch::*;

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
/// Uses cache if available, otherwise downloads fresh (if HTTP fetching enabled).
#[cfg(feature = "icon-fetch")]
pub async fn fetch_icons_async(
    source: &IconSource,
    cache_dir: &Path,
) -> crate::Result<IconMetadata> {
    let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
    fetcher.fetch(false).await
}

/// Force fetch icons (ignores cache, async).
#[cfg(feature = "icon-fetch")]
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
    // For Custom source, use local file loading
    if *source == IconSource::Custom {
        let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
        return fetcher.load_from_directory();
    }

    // Try to load from cache first
    let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
    if fetcher.is_cached() {
        return fetcher.load_cached();
    }

    #[cfg(feature = "icon-fetch")]
    {
        // Try to use tokio runtime if available
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(fetch_icons_async(source, cache_dir)),
            Err(_) => {
                // Create a new runtime
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| crate::TairitsuPackagerError::IconFetchError(e.to_string()))?;
                rt.block_on(fetch_icons_async(source, cache_dir))
            }
        }
    }

    #[cfg(not(feature = "icon-fetch"))]
    {
        warn!(
            "HTTP icon fetching not available for {:?}. \
             Enable 'icon-fetch' feature or use IconSource::Custom for local files.",
            source
        );
        Ok(IconMetadata::default())
    }
}

/// Force fetch icons (synchronous wrapper).
pub fn force_fetch_icons(source: &IconSource, cache_dir: &Path) -> crate::Result<IconMetadata> {
    #[cfg(feature = "icon-fetch")]
    {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle.block_on(force_fetch_icons_async(source, cache_dir)),
            Err(_) => {
                let rt = tokio::runtime::Runtime::new()
                    .map_err(|e| crate::TairitsuPackagerError::IconFetchError(e.to_string()))?;
                rt.block_on(force_fetch_icons_async(source, cache_dir))
            }
        }
    }

    #[cfg(not(feature = "icon-fetch"))]
    {
        let fetcher = IconFetcher::new(cache_dir.to_path_buf(), *source);
        fetcher.load_from_directory()
    }
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
        assert_eq!(path, Some("M4 4h16v16H4V4zm2 2v12h12V6H6z".to_string()));
    }

    #[test]
    fn test_extract_svg_path_no_path() {
        let svg =
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><!-- empty --></svg>"#;
        let path = extract_svg_path(svg);
        assert_eq!(path, None);
    }

    #[test]
    fn test_icon_fetcher_paths() {
        let fetcher = IconFetcher::new(PathBuf::from("/tmp/cache"), IconSource::Mdi);

        assert_eq!(fetcher.source_cache_dir(), PathBuf::from("/tmp/cache/mdi"));
        assert_eq!(
            fetcher.metadata_path(),
            PathBuf::from("/tmp/cache/mdi/metadata.json")
        );
        assert_eq!(fetcher.svg_dir(), PathBuf::from("/tmp/cache/mdi/svg"));
    }

    #[test]
    fn test_icon_fetcher_with_version() {
        let fetcher =
            IconFetcher::new(PathBuf::from("/tmp/cache"), IconSource::Mdi).with_version("7.3.67");

        assert_eq!(fetcher.version, "7.3.67");
    }
}
