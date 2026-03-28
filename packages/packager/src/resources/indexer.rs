//! Resource indexer implementation.
//!
//! Provides the `ResourceIndexer` struct for scanning directories and building
//! a resource index with content hashes for cache busting.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use super::{INDEX_FILE, RESOURCE_DIR};
use crate::resources::{ScssResource, SvgResource};

/// Resource index containing all discovered resources
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceIndex {
    /// SCSS files discovered
    pub scss: Vec<ScssResource>,
    /// SVG files discovered
    pub svg: Vec<SvgResource>,
}

impl ResourceIndex {
    /// Create a new empty resource index
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the index is empty
    pub fn is_empty(&self) -> bool {
        self.scss.is_empty() && self.svg.is_empty()
    }

    /// Get total count of resources
    pub fn count(&self) -> usize {
        self.scss.len() + self.svg.len()
    }

    /// Load index from a file
    pub fn load(path: &Path) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let index: Self = serde_json::from_str(&content)?;
        Ok(index)
    }

    /// Save index to a file
    pub fn save(&self, path: &Path) -> crate::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save index to target directory
    ///
    /// Saves to `{target_dir}/tairitsu/resources/index.json`
    pub fn save_to_target(&self, target_dir: &Path) -> crate::Result<PathBuf> {
        let output_dir = target_dir.join(RESOURCE_DIR);
        let output_path = output_dir.join(INDEX_FILE);
        self.save(&output_path)?;
        Ok(output_path)
    }

    /// Merge another index into this one
    pub fn merge(&mut self, other: ResourceIndex) {
        self.scss.extend(other.scss);
        self.svg.extend(other.svg);
    }
}

/// Resource indexer that scans directories for resources
pub struct ResourceIndexer {
    /// Project root directory
    root: PathBuf,
    /// Directories to exclude from scanning
    exclude_dirs: Vec<String>,
    /// Whether to include hidden files
    include_hidden: bool,
}

impl ResourceIndexer {
    /// Create a new resource indexer for the given root directory
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            exclude_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "dist".to_string(),
            ],
            include_hidden: false,
        }
    }

    /// Add a directory to exclude from scanning
    pub fn exclude(mut self, dir: impl Into<String>) -> Self {
        self.exclude_dirs.push(dir.into());
        self
    }

    /// Set whether to include hidden files
    pub fn include_hidden(mut self, include: bool) -> Self {
        self.include_hidden = include;
        self
    }

    /// Scan for all resources and build an index
    pub fn scan(&self) -> crate::Result<ResourceIndex> {
        let mut index = ResourceIndex::new();

        info!("Scanning for resources in {}", self.root.display());

        // Scan for SCSS files
        let scss_files = self.scan_files("scss")?;
        for file in scss_files {
            match self.process_scss_file(&file) {
                Ok(resource) => {
                    debug!("Found SCSS: {} -> {}", resource.source, resource.hash);
                    index.scss.push(resource);
                }
                Err(e) => {
                    warn!("Failed to process SCSS file {}: {}", file.display(), e);
                }
            }
        }

        // Scan for SVG files
        let svg_files = self.scan_files("svg")?;
        for file in svg_files {
            match self.process_svg_file(&file) {
                Ok(resource) => {
                    debug!(
                        "Found SVG: {} -> {} ({})",
                        resource.source, resource.hash, resource.id
                    );
                    index.svg.push(resource);
                }
                Err(e) => {
                    warn!("Failed to process SVG file {}: {}", file.display(), e);
                }
            }
        }

        info!(
            "Indexed {} SCSS files and {} SVG files",
            index.scss.len(),
            index.svg.len()
        );

        Ok(index)
    }

    /// Scan for files with a specific extension
    fn scan_files(&self, extension: &str) -> crate::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        self.scan_dir_recursive(&self.root.clone(), extension, &mut files)?;
        Ok(files)
    }

    /// Recursively scan a directory for files
    fn scan_dir_recursive(
        &self,
        dir: &Path,
        extension: &str,
        files: &mut Vec<PathBuf>,
    ) -> crate::Result<()> {
        if !dir.exists() || !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files if configured
            if !self.include_hidden
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                && name.starts_with('.')
            {
                continue;
            }

            if path.is_dir() {
                // Check if directory should be excluded
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && self.exclude_dirs.contains(&name.to_string())
                {
                    continue;
                }
                self.scan_dir_recursive(&path, extension, files)?;
            } else if path.extension().map(|e| e == extension).unwrap_or(false) {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Process a SCSS file and create a resource entry
    fn process_scss_file(&self, path: &Path) -> crate::Result<ScssResource> {
        let content = std::fs::read(path)?;
        let hash = compute_hash(&content);
        let source = path_relative_to(&self.root, path)?;

        // Generate output filename with hash
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("styles");
        let output = format!("{}.{}.css", stem, hash);

        Ok(ScssResource {
            source,
            hash,
            output,
        })
    }

    /// Process an SVG file and create a resource entry
    fn process_svg_file(&self, path: &Path) -> crate::Result<SvgResource> {
        let content = std::fs::read(path)?;
        let hash = compute_hash(&content);
        let source = path_relative_to(&self.root, path)?;

        // Extract ID from filename (without extension)
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(SvgResource { source, hash, id })
    }

    /// Index resources and save to target directory
    pub fn index_to_target(&self, target_dir: &Path) -> crate::Result<(ResourceIndex, PathBuf)> {
        let index = self.scan()?;
        let output_path = index.save_to_target(target_dir)?;
        Ok((index, output_path))
    }
}

/// Compute SHA-256 hash of content and return first 8 characters
pub fn compute_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result).chars().take(8).collect()
}

/// Get relative path from root to target
fn path_relative_to(root: &Path, target: &Path) -> crate::Result<String> {
    let relative = target.strip_prefix(root).map_err(|_| {
        crate::TairitsuPackagerError::ResourceError(format!(
            "Path {} is not relative to {}",
            target.display(),
            root.display()
        ))
    })?;
    Ok(relative.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_hash() {
        let content = b"hello world";
        let hash = compute_hash(content);
        assert_eq!(hash.len(), 8);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_resource_index_empty() {
        let index = ResourceIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_resource_index_merge() {
        let mut index1 = ResourceIndex::new();
        let mut index2 = ResourceIndex::new();

        index1.scss.push(ScssResource {
            source: "a.scss".to_string(),
            hash: "12345678".to_string(),
            output: "a.12345678.css".to_string(),
        });

        index2.svg.push(SvgResource {
            source: "b.svg".to_string(),
            hash: "abcdefgh".to_string(),
            id: "b".to_string(),
        });

        index1.merge(index2);
        assert_eq!(index1.scss.len(), 1);
        assert_eq!(index1.svg.len(), 1);
    }
}
