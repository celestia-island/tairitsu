//! Local cache management for WIT packages.
//!
//! The cache lives at `<target_dir>/tairitsu-wit/<namespace>/<name>/<version>/`
//! and contains WIT source files plus a `manifest.json` with content hashes for
//! integrity verification.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tracing::debug;

use crate::resolver::PackageSpec;

/// A single entry in the local cache.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The directory on disk that contains WIT files.
    pub wit_dir: PathBuf,
}

/// Manifest stored alongside WIT files for integrity checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Package identifier.
    pub id: String,
    /// Map from filename to hex-encoded SHA-256 hash.
    pub file_hashes: HashMap<String, String>,
}

/// Manages the `target/tairitsu-wit` cache directory.
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// Create a cache pointing at `root` (created lazily on first write).
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_owned(),
        }
    }

    /// Path to the cache directory for a specific package.
    pub fn package_dir(&self, spec: &PackageSpec) -> PathBuf {
        self.root.join(spec.cache_rel_path())
    }

    /// Check whether `spec` is present in the cache.
    ///
    /// Returns `None` if not cached, or `Some(CacheEntry)` if cached and the
    /// manifest integrity check passes.
    pub fn lookup(&self, spec: &PackageSpec) -> Result<Option<CacheEntry>> {
        let dir = self.package_dir(spec);
        if !dir.exists() {
            return Ok(None);
        }

        let manifest_path = dir.join("manifest.json");
        if !manifest_path.exists() {
            debug!(
                "Cache directory exists but manifest is missing at {}; treating as cache miss",
                manifest_path.display()
            );
            return Ok(None);
        }

        // Load and verify manifest.
        let manifest_bytes = std::fs::read(&manifest_path)
            .with_context(|| format!("Reading cache manifest at {}", manifest_path.display()))?;
        let manifest: CacheManifest = serde_json::from_slice(&manifest_bytes)
            .with_context(|| format!("Parsing cache manifest at {}", manifest_path.display()))?;

        for (filename, expected_hash) in &manifest.file_hashes {
            let file_path = dir.join(filename);
            let bytes = std::fs::read(&file_path)
                .with_context(|| format!("Reading cached WIT file {}", file_path.display()))?;
            let actual_hash = hex::encode(Sha256::digest(&bytes));
            if actual_hash != *expected_hash {
                debug!(
                    "Hash mismatch for {} in cache (expected {}, got {}); treating as cache miss",
                    filename, expected_hash, actual_hash
                );
                return Ok(None);
            }
        }

        Ok(Some(CacheEntry { wit_dir: dir }))
    }

    /// Store a set of WIT files in the cache and write a manifest.
    ///
    /// `files` is a map from filename (e.g. `dom.wit`) to file content bytes.
    pub fn store(&self, spec: &PackageSpec, files: HashMap<String, Vec<u8>>) -> Result<CacheEntry> {
        let dir = self.package_dir(spec);
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("Creating cache directory {}", dir.display()))?;

        let mut file_hashes = HashMap::new();
        for (filename, content) in &files {
            let hash = hex::encode(Sha256::digest(content));
            let dest = dir.join(filename);
            std::fs::write(&dest, content)
                .with_context(|| format!("Writing cached file {}", dest.display()))?;
            file_hashes.insert(filename.clone(), hash);
        }

        let manifest = CacheManifest {
            id: spec.id(),
            file_hashes,
        };
        let manifest_path = dir.join("manifest.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_vec_pretty(&manifest).context("Serializing cache manifest")?,
        )
        .with_context(|| format!("Writing cache manifest at {}", manifest_path.display()))?;

        Ok(CacheEntry { wit_dir: dir })
    }

    /// Remove all cached data for a specific package version.
    pub fn evict(&self, spec: &PackageSpec) -> Result<()> {
        let dir = self.package_dir(spec);
        if dir.exists() {
            std::fs::remove_dir_all(&dir)
                .with_context(|| format!("Evicting cache entry at {}", dir.display()))?;
        }
        Ok(())
    }

    /// List all cached package identifiers.
    pub fn list(&self) -> Result<Vec<String>> {
        if !self.root.exists() {
            return Ok(vec![]);
        }

        let mut ids = Vec::new();
        // Walk namespace / name / version tree.
        for ns_entry in std::fs::read_dir(&self.root)? {
            let ns_entry = ns_entry?;
            for name_entry in std::fs::read_dir(ns_entry.path())? {
                let name_entry = name_entry?;
                for ver_entry in std::fs::read_dir(name_entry.path())? {
                    let ver_entry = ver_entry?;
                    let manifest_path = ver_entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        let bytes = std::fs::read(&manifest_path)?;
                        if let Ok(m) = serde_json::from_slice::<CacheManifest>(&bytes) {
                            ids.push(m.id);
                        }
                    }
                }
            }
        }
        ids.sort();
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_store_and_lookup() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = Cache::new(tmp.path());

        let spec = PackageSpec::parse("tairitsu-browser:dom@0.1.0").unwrap();
        let mut files = HashMap::new();
        files.insert(
            "dom.wit".to_owned(),
            b"package tairitsu-browser:dom@0.1.0;".to_vec(),
        );

        cache.store(&spec, files).unwrap();

        let entry = cache.lookup(&spec).unwrap();
        assert!(entry.is_some(), "Expected cache hit after store");
    }

    #[test]
    fn cache_miss_for_unknown_package() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = Cache::new(tmp.path());
        let spec = PackageSpec::parse("tairitsu-browser:canvas@99.0.0").unwrap();
        let result = cache.lookup(&spec).unwrap();
        assert!(result.is_none(), "Expected cache miss for unknown package");
    }

    #[test]
    fn cache_list_returns_stored_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = Cache::new(tmp.path());

        let spec1 = PackageSpec::parse("tairitsu-browser:dom@0.1.0").unwrap();
        let spec2 = PackageSpec::parse("tairitsu-browser:events@0.1.0").unwrap();

        let mut files = HashMap::new();
        files.insert(
            "dom.wit".to_owned(),
            b"package tairitsu-browser:dom@0.1.0;".to_vec(),
        );
        cache.store(&spec1, files.clone()).unwrap();

        files.clear();
        files.insert(
            "events.wit".to_owned(),
            b"package tairitsu-browser:events@0.1.0;".to_vec(),
        );
        cache.store(&spec2, files).unwrap();

        let ids = cache.list().unwrap();
        assert!(ids.contains(&"tairitsu-browser:dom@0.1.0".to_owned()));
        assert!(ids.contains(&"tairitsu-browser:events@0.1.0".to_owned()));
    }
}
