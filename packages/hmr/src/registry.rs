//! Module registry for HMR
//!
//! Tracks loaded modules and their dependencies for hot replacement.

use crate::protocol::ModuleState;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Information about a loaded module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Unique module identifier
    pub id: String,
    /// Module path or URL
    pub path: String,
    /// Current state of the module
    pub state: ModuleState,
    /// Dependencies of this module
    pub dependencies: HashSet<String>,
    /// Modules that depend on this one
    pub dependents: HashSet<String>,
    /// Timestamp when module was loaded
    pub loaded_at: u64,
    /// Version hash for change detection
    pub version: String,
}

impl ModuleInfo {
    /// Create a new module info
    pub fn new(path: impl Into<String>) -> Self {
        let id = Uuid::new_v4().to_string();
        ModuleInfo {
            id,
            path: path.into(),
            state: ModuleState::Loading,
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            loaded_at: 0,
            version: String::new(),
        }
    }

    /// Add a dependency
    pub fn add_dependency(&mut self, dep_id: impl Into<String>) {
        self.dependencies.insert(dep_id.into());
    }

    /// Add a dependent
    pub fn add_dependent(&mut self, dep_id: impl Into<String>) {
        self.dependents.insert(dep_id.into());
    }

    /// Mark as loaded
    pub fn mark_loaded(&mut self, version: impl Into<String>) {
        self.state = ModuleState::Loaded;
        self.version = version.into();
        self.loaded_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.state = ModuleState::Failed {
            error: error.into(),
        };
    }

    /// Mark as disposed
    pub fn mark_disposed(&mut self) {
        self.state = ModuleState::Disposed;
    }

    /// Check if module is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ModuleState::Loaded)
    }
}

/// Registry for tracking modules and their relationships
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    modules: Arc<RwLock<HashMap<String, ModuleInfo>>>,
    path_to_id: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        ModuleRegistry {
            modules: Arc::new(RwLock::new(HashMap::new())),
            path_to_id: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new module
    pub fn register(&self, path: impl Into<String>) -> String {
        let path = path.into();
        let info = ModuleInfo::new(&path);
        let id = info.id.clone();

        let mut modules = self.modules.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();

        path_to_id.insert(path.clone(), id.clone());
        modules.insert(id.clone(), info);

        id
    }

    /// Get module info by ID
    pub fn get(&self, id: &str) -> Option<ModuleInfo> {
        let modules = self.modules.read().unwrap();
        modules.get(id).cloned()
    }

    /// Get module ID by path
    pub fn get_id_by_path(&self, path: &str) -> Option<String> {
        let path_to_id = self.path_to_id.read().unwrap();
        path_to_id.get(path).cloned()
    }

    /// Update module state
    pub fn update_state(&self, id: &str, state: ModuleState) -> Result<(), String> {
        let mut modules = self.modules.write().unwrap();
        let module = modules.get_mut(id).ok_or("Module not found")?;
        module.state = state;
        Ok(())
    }

    /// Mark module as loaded with version
    pub fn mark_loaded(&self, id: &str, version: impl Into<String>) -> Result<(), String> {
        let mut modules = self.modules.write().unwrap();
        let module = modules.get_mut(id).ok_or("Module not found")?;
        module.mark_loaded(version);
        Ok(())
    }

    /// Mark module as failed
    pub fn mark_failed(&self, id: &str, error: impl Into<String>) -> Result<(), String> {
        let mut modules = self.modules.write().unwrap();
        let module = modules.get_mut(id).ok_or("Module not found")?;
        module.mark_failed(error);
        Ok(())
    }

    /// Add dependency relationship
    pub fn add_dependency(&self, module_id: &str, dep_id: &str) -> Result<(), String> {
        let mut modules = self.modules.write().unwrap();

        let module = modules.get_mut(module_id).ok_or("Module not found")?;
        module.add_dependency(dep_id);

        let dep = modules.get_mut(dep_id).ok_or("Dependency not found")?;
        dep.add_dependent(module_id);

        Ok(())
    }

    /// Get all dependents of a module (transitive)
    pub fn get_all_dependents(&self, id: &str) -> HashSet<String> {
        let mut dependents = HashSet::new();
        let mut to_visit = vec![id.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(module) = self.get(&current) {
                for dep in module.dependents {
                    if dependents.insert(dep.clone()) {
                        to_visit.push(dep);
                    }
                }
            }
        }

        dependents
    }

    /// Get all dependencies of a module (transitive)
    pub fn get_all_dependencies(&self, id: &str) -> HashSet<String> {
        let mut dependencies = HashSet::new();
        let mut to_visit = vec![id.to_string()];

        while let Some(current) = to_visit.pop() {
            if let Some(module) = self.get(&current) {
                for dep in module.dependencies {
                    if dependencies.insert(dep.clone()) {
                        to_visit.push(dep);
                    }
                }
            }
        }

        dependencies
    }

    /// Remove a module from registry
    pub fn remove(&self, id: &str) -> Result<(), String> {
        let mut modules = self.modules.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();

        let module = modules.get(id).ok_or("Module not found")?;
        path_to_id.remove(&module.path);
        modules.remove(id);

        Ok(())
    }

    /// Clear all modules
    pub fn clear(&self) {
        let mut modules = self.modules.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();

        modules.clear();
        path_to_id.clear();
    }

    /// Get count of registered modules
    pub fn count(&self) -> usize {
        let modules = self.modules.read().unwrap();
        modules.len()
    }

    /// List all module IDs
    pub fn list_ids(&self) -> Vec<String> {
        let modules = self.modules.read().unwrap();
        modules.keys().cloned().collect()
    }

    /// Find modules by state
    pub fn find_by_state(&self, state: ModuleState) -> Vec<ModuleInfo> {
        let modules = self.modules.read().unwrap();
        modules
            .values()
            .filter(|m| m.state == state)
            .cloned()
            .collect()
    }

    /// Create a snapshot of the registry for transmission
    pub fn snapshot(&self) -> Vec<ModuleInfo> {
        let modules = self.modules.read().unwrap();
        modules.values().cloned().collect()
    }

    /// Restore registry from snapshot
    pub fn restore(&self, snapshot: Vec<ModuleInfo>) {
        let mut modules = self.modules.write().unwrap();
        let mut path_to_id = self.path_to_id.write().unwrap();

        modules.clear();
        path_to_id.clear();

        for info in snapshot {
            path_to_id.insert(info.path.clone(), info.id.clone());
            modules.insert(info.id.clone(), info);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_module() {
        let registry = ModuleRegistry::new();
        let id = registry.register("/path/to/module.js");

        let module = registry.get(&id).unwrap();
        assert_eq!(module.path, "/path/to/module.js");
        assert!(matches!(module.state, ModuleState::Loading));
    }

    #[test]
    fn test_get_id_by_path() {
        let registry = ModuleRegistry::new();
        let id = registry.register("/path/to/module.js");

        assert_eq!(registry.get_id_by_path("/path/to/module.js"), Some(id));
    }

    #[test]
    fn test_mark_loaded() {
        let registry = ModuleRegistry::new();
        let id = registry.register("/path/to/module.js");

        registry.mark_loaded(&id, "v1").unwrap();

        let module = registry.get(&id).unwrap();
        assert!(module.is_active());
        assert_eq!(module.version, "v1");
    }

    #[test]
    fn test_dependencies() {
        let registry = ModuleRegistry::new();
        let id1 = registry.register("/module1.js");
        let id2 = registry.register("/module2.js");

        registry.add_dependency(&id1, &id2).unwrap();

        let module1 = registry.get(&id1).unwrap();
        assert!(module1.dependencies.contains(&id2));

        let module2 = registry.get(&id2).unwrap();
        assert!(module2.dependents.contains(&id1));
    }

    #[test]
    fn test_transitive_dependents() {
        let registry = ModuleRegistry::new();
        let id1 = registry.register("/module1.js");
        let id2 = registry.register("/module2.js");
        let id3 = registry.register("/module3.js");

        // id3 depends on id2, id2 depends on id1
        registry.add_dependency(&id2, &id1).unwrap();
        registry.add_dependency(&id3, &id2).unwrap();

        let dependents = registry.get_all_dependents(&id1);
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&id2));
        assert!(dependents.contains(&id3));
    }

    #[test]
    fn test_find_by_state() {
        let registry = ModuleRegistry::new();
        let id1 = registry.register("/module1.js");
        let _id2 = registry.register("/module2.js");

        registry.mark_loaded(&id1, "v1").unwrap();

        let loaded = registry.find_by_state(ModuleState::Loaded);
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id1);
    }

    #[test]
    fn test_snapshot_restore() {
        let registry1 = ModuleRegistry::new();
        let id = registry1.register("/module.js");
        registry1.mark_loaded(&id, "v1").unwrap();

        let snapshot = registry1.snapshot();

        let registry2 = ModuleRegistry::new();
        registry2.restore(snapshot);

        let module = registry2.get(&id).unwrap();
        assert_eq!(module.path, "/module.js");
        assert!(module.is_active());
    }
}
