use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::Result;

#[derive(Debug, Clone)]
pub struct PluginWitFile {
    pub path: PathBuf,
    pub interface_name: String,
    pub source: String,
}

#[derive(Debug, Default)]
pub struct PluginWitRegistry {
    pub plugins: Vec<PluginWitFile>,
}

impl PluginWitRegistry {
    pub fn scan_project_dir(manifest_dir: &Path) -> Result<Self> {
        let mut registry = Self::default();
        let wit_dir = manifest_dir.join("wit");

        if !wit_dir.is_dir() {
            return Ok(registry);
        }

        for entry in walkdir::WalkDir::new(&wit_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if path.extension().and_then(|e| e.to_str()) == Some("wit") {
                let source = std::fs::read_to_string(&path)?;
                let interface_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                registry.plugins.push(PluginWitFile {
                    path,
                    interface_name,
                    source,
                });
            }
        }

        Ok(registry)
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }

    pub fn plugin_count(&self) -> usize {
        self.plugins.len()
    }

    pub fn merge_into_composed_dir(
        &self,
        composed_dir: &Path,
    ) -> Result<HashMap<String, PathBuf>> {
        if self.is_empty() {
            return Ok(HashMap::new());
        }

        let plugin_dir = composed_dir.join("_plugins");
        std::fs::create_dir_all(&plugin_dir)?;

        let mut added = HashMap::new();
        for plugin in &self.plugins {
            let dest = plugin_dir.join(
                plugin
                    .path
                    .file_name()
                    .unwrap_or_default(),
            );
            std::fs::write(&dest, &plugin.source)?;
            added.insert(plugin.interface_name.clone(), dest);
        }

        Ok(added)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_scan_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let registry = PluginWitRegistry::scan_project_dir(tmp.path()).unwrap();
        assert!(registry.is_empty());
    }

    #[test]
    fn test_scan_with_wit_files() {
        let tmp = tempfile::tempdir().unwrap();
        let wit_dir = tmp.path().join("wit");
        fs::create_dir_all(&wit_dir).unwrap();
        fs::write(
            wit_dir.join("echarts-helpers.wit"),
            "package tairitsu-browser:echarts;\n\ninterface echarts-helpers {\n    echarts-init: func(canvas-id: string, option-json: string) -> u64;\n}",
        ).unwrap();

        let registry = PluginWitRegistry::scan_project_dir(tmp.path()).unwrap();
        assert_eq!(registry.plugin_count(), 1);
        assert_eq!(registry.plugins[0].interface_name, "echarts-helpers");
    }

    #[test]
    fn test_merge_into_composed_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let wit_dir = tmp.path().join("wit");
        fs::create_dir_all(&wit_dir).unwrap();
        fs::write(
            wit_dir.join("test.wit"),
            "interface test { hello: func() -> string; }",
        )
        .unwrap();

        let registry = PluginWitRegistry::scan_project_dir(tmp.path()).unwrap();
        let composed_dir = tmp.path().join("composed");
        fs::create_dir_all(&composed_dir).unwrap();

        let added = registry.merge_into_composed_dir(&composed_dir).unwrap();
        assert_eq!(added.len(), 1);
        assert!(composed_dir.join("_plugins/test.wit").exists());
    }
}
