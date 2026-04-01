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
    fn test_compute_hash_consistent() {
        let content = b"consistent content";
        let hash1 = compute_hash(content);
        let hash2 = compute_hash(content);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_different_content() {
        let hash1 = compute_hash(b"content1");
        let hash2 = compute_hash(b"content2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_compute_hash_empty() {
        let content = b"";
        let hash = compute_hash(content);
        assert_eq!(hash.len(), 8);
    }

    #[test]
    fn test_resource_index_empty() {
        let index = ResourceIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_resource_index_with_scss() {
        let mut index = ResourceIndex::new();
        index.scss.push(ScssResource {
            source: "styles/main.scss".to_string(),
            hash: "abc12345".to_string(),
            output: "main.abc12345.css".to_string(),
        });

        assert!(!index.is_empty());
        assert_eq!(index.count(), 1);
        assert_eq!(index.scss.len(), 1);
    }

    #[test]
    fn test_resource_index_with_svg() {
        let mut index = ResourceIndex::new();
        index.svg.push(SvgResource {
            source: "icons/home.svg".to_string(),
            hash: "def67890".to_string(),
            id: "home".to_string(),
        });

        assert!(!index.is_empty());
        assert_eq!(index.count(), 1);
        assert_eq!(index.svg.len(), 1);
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
        assert_eq!(index1.count(), 2);
    }

    #[test]
    fn test_resource_index_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index_path = temp_dir.path().join("index.json");

        let mut index = ResourceIndex::new();
        index.scss.push(ScssResource {
            source: "test.scss".to_string(),
            hash: "testhash".to_string(),
            output: "test.testhash.css".to_string(),
        });
        index.svg.push(SvgResource {
            source: "test.svg".to_string(),
            hash: "svghash".to_string(),
            id: "test".to_string(),
        });

        // Save
        index.save(&index_path).unwrap();
        assert!(index_path.exists());

        // Load
        let loaded = ResourceIndex::load(&index_path).unwrap();
        assert_eq!(loaded.scss.len(), 1);
        assert_eq!(loaded.svg.len(), 1);
        assert_eq!(loaded.scss[0].source, "test.scss");
        assert_eq!(loaded.svg[0].id, "test");
    }

    #[test]
    fn test_resource_index_save_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dir/index.json");

        let index = ResourceIndex::new();
        index.save(&nested_path).unwrap();

        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());
    }

    #[test]
    fn test_resource_index_save_to_target() {
        let temp_dir = tempfile::tempdir().unwrap();

        let mut index = ResourceIndex::new();
        index.scss.push(ScssResource {
            source: "main.scss".to_string(),
            hash: "aaaa1111".to_string(),
            output: "main.aaaa1111.css".to_string(),
        });

        let output_path = index.save_to_target(temp_dir.path()).unwrap();

        assert!(output_path.exists());
        assert!(output_path.ends_with("tairitsu/resources/index.json"));
    }

    #[test]
    fn test_resource_indexer_new() {
        let temp_dir = tempfile::tempdir().unwrap();
        let indexer = ResourceIndexer::new(temp_dir.path());

        assert_eq!(indexer.root, temp_dir.path());
        assert_eq!(indexer.exclude_dirs.len(), 4);
        assert!(indexer.exclude_dirs.contains(&"target".to_string()));
        assert!(indexer.exclude_dirs.contains(&"node_modules".to_string()));
        assert!(indexer.exclude_dirs.contains(&".git".to_string()));
        assert!(indexer.exclude_dirs.contains(&"dist".to_string()));
        assert!(!indexer.include_hidden);
    }

    #[test]
    fn test_resource_indexer_exclude() {
        let temp_dir = tempfile::tempdir().unwrap();
        let indexer = ResourceIndexer::new(temp_dir.path()).exclude("custom_exclude");

        assert!(indexer.exclude_dirs.contains(&"custom_exclude".to_string()));
    }

    #[test]
    fn test_resource_indexer_include_hidden() {
        let temp_dir = tempfile::tempdir().unwrap();
        let indexer = ResourceIndexer::new(temp_dir.path()).include_hidden(true);

        assert!(indexer.include_hidden);
    }

    #[test]
    fn test_resource_indexer_scan_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let indexer = ResourceIndexer::new(temp_dir.path());

        let index = indexer.scan().unwrap();
        assert!(index.is_empty());
        assert_eq!(index.count(), 0);
    }

    #[test]
    fn test_resource_indexer_scan_with_scss_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create SCSS directory and file
        let scss_dir = temp_dir.path().join("scss");
        std::fs::create_dir_all(&scss_dir).unwrap();
        std::fs::write(scss_dir.join("main.scss"), b".test { color: red; }").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let index = indexer.scan().unwrap();

        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, "scss/main.scss");
        assert!(index.scss[0].output.starts_with("main."));
        assert!(!index.scss[0].hash.is_empty());
    }

    #[test]
    fn test_resource_indexer_scan_with_svg_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create SVG directory and file
        let svg_dir = temp_dir.path().join("icons");
        std::fs::create_dir_all(&svg_dir).unwrap();
        std::fs::write(svg_dir.join("home.svg"), b"<svg></svg>").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let index = indexer.scan().unwrap();

        assert_eq!(index.svg.len(), 1);
        assert_eq!(index.svg[0].source, "icons/home.svg");
        assert_eq!(index.svg[0].id, "home");
        assert!(!index.svg[0].hash.is_empty());
    }

    #[test]
    fn test_resource_indexer_scan_excludes_directories() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create files in excluded directories
        let target_dir = temp_dir.path().join("target");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("test.scss"), b".test {}").unwrap();

        let node_modules_dir = temp_dir.path().join("node_modules");
        std::fs::create_dir_all(&node_modules_dir).unwrap();
        std::fs::write(node_modules_dir.join("test.scss"), b".test {}").unwrap();

        // Create file in non-excluded directory
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.scss"), b".test {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let index = indexer.scan().unwrap();

        // Should only find the file in src, not in target or node_modules
        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, "src/main.scss");
    }

    #[test]
    fn test_resource_indexer_scan_hidden_files() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create hidden file
        std::fs::write(temp_dir.path().join(".hidden.scss"), b".test {}").unwrap();

        // Create regular file
        std::fs::write(temp_dir.path().join("visible.scss"), b".test {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let index = indexer.scan().unwrap();

        // Should only find visible file
        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, "visible.scss");
    }

    #[test]
    fn test_resource_indexer_scan_hidden_files_when_enabled() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create hidden file
        std::fs::write(temp_dir.path().join(".hidden.scss"), b".test {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path()).include_hidden(true);
        let index = indexer.scan().unwrap();

        // Should find hidden file
        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, ".hidden.scss");
    }

    #[test]
    fn test_resource_indexer_scan_nested_directories() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create nested structure
        let nested_dir = temp_dir.path().join("styles/components");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join("button.scss"), b".btn {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let index = indexer.scan().unwrap();

        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, "styles/components/button.scss");
    }

    #[test]
    fn test_resource_indexer_index_to_target() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create a test file
        std::fs::write(temp_dir.path().join("test.scss"), b".test {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path());
        let (index, output_path) = indexer.index_to_target(target_dir.path()).unwrap();

        assert_eq!(index.scss.len(), 1);
        assert!(output_path.exists());
        assert!(output_path.starts_with(target_dir));
    }

    #[test]
    fn test_resource_indexer_custom_exclude() {
        let temp_dir = tempfile::tempdir().unwrap();

        // Create directory to be excluded
        let exclude_dir = temp_dir.path().join("vendor");
        std::fs::create_dir_all(&exclude_dir).unwrap();
        std::fs::write(exclude_dir.join("test.scss"), b".test {}").unwrap();

        // Create regular directory
        let src_dir = temp_dir.path().join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.scss"), b".test {}").unwrap();

        let indexer = ResourceIndexer::new(temp_dir.path()).exclude("vendor");
        let index = indexer.scan().unwrap();

        // Should only find the file in src
        assert_eq!(index.scss.len(), 1);
        assert_eq!(index.scss[0].source, "src/main.scss");
    }

    #[test]
    fn test_path_relative_to() {
        let root = tempfile::tempdir().unwrap();
        let file_path = root.path().join("subdir").join("file.txt");

        // Create the file so it exists
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        std::fs::write(&file_path, b"content").unwrap();

        let relative = path_relative_to(root.path(), &file_path).unwrap();
        assert_eq!(relative, "subdir/file.txt");
    }

    #[test]
    fn test_path_relative_to_not_relative() {
        let root = tempfile::tempdir().unwrap();
        let other_root = tempfile::tempdir().unwrap();
        let file_path = other_root.path().join("file.txt");

        let result = path_relative_to(root.path(), &file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_resource_index_serialization() {
        let index = ResourceIndex {
            scss: vec![ScssResource {
                source: "test.scss".to_string(),
                hash: "12345678".to_string(),
                output: "test.12345678.css".to_string(),
            }],
            svg: vec![SvgResource {
                source: "test.svg".to_string(),
                hash: "abcdefgh".to_string(),
                id: "test".to_string(),
            }],
        };

        let json = serde_json::to_string(&index).unwrap();
        assert!(json.contains("\"scss\""));
        assert!(json.contains("\"svg\""));
        assert!(json.contains("test.scss"));

        let deserialized: ResourceIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scss.len(), 1);
        assert_eq!(deserialized.svg.len(), 1);
    }
}
