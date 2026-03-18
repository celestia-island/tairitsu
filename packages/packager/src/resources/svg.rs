//! SVG resource handling.
//!
//! Provides structures and utilities for managing SVG file resources
//! with content hashing for cache busting.

use serde::{Deserialize, Serialize};

use super::ResourceEntry;

/// An SVG resource entry in the index
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SvgResource {
    /// Source path relative to project root
    pub source: String,
    /// Content hash (first 8 characters of SHA-256)
    pub hash: String,
    /// Icon ID derived from filename (without extension)
    pub id: String,
}

impl SvgResource {
    /// Create a new SVG resource
    pub fn new(source: impl Into<String>, hash: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            hash: hash.into(),
            id: id.into(),
        }
    }

    /// Get the output path relative to the resource output directory
    pub fn output_path(&self) -> String {
        format!("icons/{}.{}.svg", self.id, self.hash)
    }

    /// Get the source directory
    pub fn source_dir(&self) -> Option<&str> {
        self.source.rfind('/').map(|i| &self.source[..i])
    }

    /// Get the source filename
    pub fn source_filename(&self) -> &str {
        self.source.rfind('/')
            .map(|i| &self.source[i + 1..])
            .unwrap_or(&self.source)
    }

    /// Convert the ID to kebab-case if not already
    pub fn normalized_id(&self) -> String {
        // SVG IDs should be valid identifiers
        self.id
            .replace('_', "-")
            .replace(' ', "-")
            .to_lowercase()
    }

    /// Get the symbol ID for use in SVG sprites
    pub fn symbol_id(&self) -> String {
        format!("icon-{}", self.normalized_id())
    }
}

impl From<SvgResource> for ResourceEntry {
    fn from(resource: SvgResource) -> Self {
        ResourceEntry::new(resource.source, resource.hash)
    }
}

/// SVG resource utilities
pub struct SvgUtils;

impl SvgUtils {
    /// Check if a file is a valid SVG file
    pub fn is_svg_file(path: &std::path::Path) -> bool {
        path.extension()
            .map(|ext| ext == "svg")
            .unwrap_or(false)
    }

    /// Extract the icon ID from a file path
    pub fn extract_id(path: &std::path::Path) -> Option<String> {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .map(|s| s.to_string())
    }

    /// Validate an SVG ID (must be a valid CSS identifier)
    pub fn is_valid_id(id: &str) -> bool {
        if id.is_empty() {
            return false;
        }

        // First character must be a letter or underscore
        let first_char = id.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' && first_char != '-' {
            return false;
        }

        // Remaining characters must be alphanumeric, underscore, or hyphen
        id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    }

    /// Normalize an ID to a valid CSS identifier
    pub fn normalize_id(id: &str) -> String {
        id.replace('_', "-")
            .replace(' ', "-")
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
            .to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_resource_creation() {
        let resource = SvgResource::new(
            "src/icons/sun.svg",
            "def45678",
            "sun"
        );
        assert_eq!(resource.source, "src/icons/sun.svg");
        assert_eq!(resource.hash, "def45678");
        assert_eq!(resource.id, "sun");
    }

    #[test]
    fn test_svg_resource_paths() {
        let resource = SvgResource::new(
            "src/icons/weather/sun.svg",
            "def45678",
            "sun"
        );
        assert_eq!(resource.source_dir(), Some("src/icons/weather"));
        assert_eq!(resource.source_filename(), "sun.svg");
        assert_eq!(resource.output_path(), "icons/sun.def45678.svg");
        assert_eq!(resource.symbol_id(), "icon-sun");
    }

    #[test]
    fn test_svg_resource_normalized_id() {
        let resource = SvgResource::new(
            "src/icons/Sun_Icon.svg",
            "def45678",
            "Sun_Icon"
        );
        assert_eq!(resource.normalized_id(), "sun-icon");
        assert_eq!(resource.symbol_id(), "icon-sun-icon");
    }

    #[test]
    fn test_svg_utils() {
        let path = std::path::Path::new("src/icons/sun.svg");
        assert!(SvgUtils::is_svg_file(path));
        assert_eq!(SvgUtils::extract_id(path), Some("sun".to_string()));
        assert!(SvgUtils::is_valid_id("sun"));
        assert!(SvgUtils::is_valid_id("sun-icon"));
        assert!(SvgUtils::is_valid_id("_private"));
        assert!(!SvgUtils::is_valid_id("123invalid"));
        assert!(!SvgUtils::is_valid_id(""));
    }

    #[test]
    fn test_svg_utils_normalize() {
        assert_eq!(SvgUtils::normalize_id("Sun_Icon"), "sun-icon");
        assert_eq!(SvgUtils::normalize_id("my icon name"), "my-icon-name");
        assert_eq!(SvgUtils::normalize_id("Icon@Home"), "icon-home");
    }
}
