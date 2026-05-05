//! SCSS resource handling.
//!
//! Provides structures and utilities for managing SCSS file resources
//! with content hashing for cache busting.

use serde::{Deserialize, Serialize};

use super::ResourceEntry;

/// A SCSS resource entry in the index
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScssResource {
    /// Source path relative to project root
    pub source: String,
    /// Content hash (first 8 characters of SHA-256)
    pub hash: String,
    /// Output filename with hash for cache busting (e.g., "main.abc123.css")
    pub output: String,
}

impl ScssResource {
    /// Create a new SCSS resource
    pub fn new(
        source: impl Into<String>,
        hash: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            source: source.into(),
            hash: hash.into(),
            output: output.into(),
        }
    }

    /// Get the output path relative to the resource output directory
    pub fn output_path(&self) -> String {
        format!("css/{}", self.output)
    }

    /// Get the source directory
    pub fn source_dir(&self) -> Option<&str> {
        self.source.rfind('/').map(|i| &self.source[..i])
    }

    /// Get the source filename
    pub fn source_filename(&self) -> &str {
        self.source
            .rfind('/')
            .map(|i| &self.source[i + 1..])
            .unwrap_or(&self.source)
    }

    /// Check if this is a partial SCSS file (starts with _)
    pub fn is_partial(&self) -> bool {
        self.source_filename().starts_with('_')
    }

    /// Get the base name without extension and hash
    pub fn base_name(&self) -> &str {
        let filename = self.source_filename();
        filename.strip_suffix(".scss").unwrap_or(filename)
    }
}

impl From<ScssResource> for ResourceEntry {
    fn from(resource: ScssResource) -> Self {
        ResourceEntry::new(resource.source, resource.hash)
    }
}

/// SCSS resource utilities
pub struct ScssUtils;

impl ScssUtils {
    /// Check if a file is a valid SCSS file
    pub fn is_scss_file(path: &std::path::Path) -> bool {
        path.extension().map(|ext| ext == "scss").unwrap_or(false)
    }

    /// Check if a SCSS file is a partial (should not be compiled directly)
    pub fn is_partial(path: &std::path::Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('_'))
            .unwrap_or(false)
    }

    /// Get the main module name from a SCSS file path
    pub fn module_name(path: &std::path::Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.trim_start_matches('_').to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scss_resource_creation() {
        let resource = ScssResource::new("src/styles/main.scss", "abc12345", "main.abc12345.css");
        assert_eq!(resource.source, "src/styles/main.scss");
        assert_eq!(resource.hash, "abc12345");
        assert_eq!(resource.output, "main.abc12345.css");
    }

    #[test]
    fn test_scss_resource_paths() {
        let resource = ScssResource::new("src/styles/main.scss", "abc12345", "main.abc12345.css");
        assert_eq!(resource.source_dir(), Some("src/styles"));
        assert_eq!(resource.source_filename(), "main.scss");
        assert_eq!(resource.base_name(), "main");
        assert!(!resource.is_partial());
    }

    #[test]
    fn test_scss_partial() {
        let resource = ScssResource::new(
            "src/styles/_variables.scss",
            "def45678",
            "_variables.def45678.css",
        );
        assert!(resource.is_partial());
        assert_eq!(resource.base_name(), "_variables");
    }

    #[test]
    fn test_scss_utils() {
        let path = std::path::Path::new("src/styles/main.scss");
        assert!(ScssUtils::is_scss_file(path));
        assert!(!ScssUtils::is_partial(path));
        assert_eq!(ScssUtils::module_name(path), Some("main".to_string()));

        let partial = std::path::Path::new("src/styles/_vars.scss");
        assert!(ScssUtils::is_partial(partial));
        assert_eq!(ScssUtils::module_name(partial), Some("vars".to_string()));
    }
}
