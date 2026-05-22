use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    pub set_name: String,
    pub version: String,
    pub source_hash: String,
    pub icon_count: usize,
    pub icons: HashMap<String, IconData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconData {
    pub path_d: String,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
}

impl CacheManifest {
    pub fn load(path: &Path) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string(self)?;
        let mut f = fs::File::create(path)?;
        f.write_all(content.as_bytes())
    }

    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_hash(&self, current_data: &[u8]) -> bool {
        let current_hash = Self::compute_hash(current_data);
        self.source_hash == current_hash
    }
}

pub struct IconCache {
    root: PathBuf,
    offline: bool,
}

impl IconCache {
    pub fn new(root: PathBuf, offline: bool) -> Self {
        Self { root, offline }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn is_offline(&self) -> bool {
        self.offline
    }

    pub fn set_dir(&self, set_name: &str, version: &str) -> PathBuf {
        self.root.join(set_name).join(version)
    }

    pub fn manifest_path(&self, set_name: &str, version: &str) -> PathBuf {
        self.set_dir(set_name, version).join("manifest.json")
    }

    pub fn svg_data_path(&self, set_name: &str, version: &str) -> PathBuf {
        self.set_dir(set_name, version).join("svg_data.dat")
    }

    pub fn font_path(&self, set_name: &str, version: &str) -> PathBuf {
        self.set_dir(set_name, version).join("subset.woff2")
    }

    pub fn load_manifest(&self, set_name: &str, version: &str) -> Option<CacheManifest> {
        let path = self.manifest_path(set_name, version);
        if path.exists() {
            CacheManifest::load(&path)
        } else {
            None
        }
    }

    pub fn save_manifest(&self, manifest: &CacheManifest) -> std::io::Result<()> {
        let path = self.manifest_path(&manifest.set_name, &manifest.version);
        manifest.save(&path)
    }

    pub fn save_svg_data(
        &self,
        set_name: &str,
        version: &str,
        entries: &[(String, String)],
    ) -> std::io::Result<PathBuf> {
        let dir = self.set_dir(set_name, version);
        fs::create_dir_all(&dir)?;
        let path = dir.join("svg_data.dat");
        let mut f = fs::File::create(&path)?;
        for (name, path_d) in entries {
            writeln!(f, "{}\t{}", name, path_d)?;
        }
        Ok(path)
    }

    pub fn load_svg_data(&self, set_name: &str, version: &str) -> std::io::Result<Vec<(String, String)>> {
        let path = self.svg_data_path(set_name, version);
        let content = fs::read_to_string(path)?;
        let mut entries = Vec::new();
        for line in content.lines() {
            if let Some((name, path_d)) = line.split_once('\t') {
                entries.push((name.to_string(), path_d.to_string()));
            }
        }
        Ok(entries)
    }

    pub fn ensure_dir(&self, set_name: &str, version: &str) -> std::io::Result<()> {
        let dir = self.set_dir(set_name, version);
        fs::create_dir_all(dir)
    }

    pub fn has_cache(&self, set_name: &str, version: &str) -> bool {
        self.manifest_path(set_name, version).exists()
    }
}

pub fn resolve_cache_root(manifest_dir: Option<&Path>) -> PathBuf {
    if let Ok(root) = std::env::var("HIKARI_ICONS_CACHE") {
        return PathBuf::from(root);
    }
    if let Some(dir) = manifest_dir {
        return dir.join("target").join("tairitsu-cache").join("icons");
    }
    if let Ok(target_dir) = std::env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(target_dir).join("tairitsu-cache").join("icons");
    }
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut dir = PathBuf::from(&manifest_dir);
        loop {
            if dir.join("Cargo.toml").exists() {
                if let Some(parent) = dir.parent() {
                    if parent.join("Cargo.toml").exists() || parent.join("Cargo.lock").exists() {
                        return parent.join("target").join("tairitsu-cache").join("icons");
                    }
                }
                return dir.join("target").join("tairitsu-cache").join("icons");
            }
            if !dir.pop() {
                break;
            }
        }
        return PathBuf::from(manifest_dir).join("target").join("tairitsu-cache").join("icons");
    }
    let base = std::env::var("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            PathBuf::from(home).join(".cache")
        });
    base.join("tairitsu-cache").join("icons")
}
