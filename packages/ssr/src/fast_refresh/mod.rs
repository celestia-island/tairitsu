//! Fast Refresh support for Tairitsu
//!
//! This module provides Fast Refresh functionality similar to React Fast Refresh,
//! allowing components to be updated during development without losing state.
//!
//! # Example
//!
//! ```rust
//! use tairitsu_ssr::fast_refresh::{sign_component, FastRefreshRuntime};
//! ```
//! // Sign your component at declaration site
//! let sig = sign_component("MyComponent", "src/app.rs", 42);
//!
//! // Register with runtime
//! let runtime = FastRefreshRuntime::new();
//! let result = runtime.register_component(sig, 1);
//! ```

mod diff;

use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex, RwLock}};
use serde::{Deserialize, Serialize};

use tairitsu_vdom::ComponentId;
pub use diff::{ComponentChange, ComponentMetadata, DiffResult, FunctionChange, HookChange, HookInfo, HookType, PropertyChange, PropertyInfo, diff_components};

/// Signature for identifying components across updates
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentSignature {
    /// Component name (e.g., "MyComponent")
    pub name: String,
    /// File path where component is defined (e.g., "src/components/button.rs")
    pub file: String,
    /// Line number where component is defined
    pub line: u32,
}

impl ComponentSignature {
    /// Create a new component signature
    pub fn new(name: impl Into<String>, file: impl Into<String>, line: u32) -> Self {
        Self {
            name: name.into(),
            file: file.into(),
            line,
        }
    }

    /// Create a signature from a function-like string
    ///
    /// Parses strings like "function MyComponent" or "const MyComponent = ()"
    pub fn parse(input: &str, file: &str) -> Option<Self> {
        let re = regex::Regex::new(r"(?:function|const|let|var)\s+(\w+)").ok()?;
        let captures = re.captures(input)?;
        let name = captures.get(1)?.as_str().to_string();
        Some(Self::new(name, file, 0))
    }

    /// Get a hash of this signature for quick comparison
    pub fn hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.name.hash(&mut hasher);
        self.file.hash(&mut hasher);
        self.line.hash(&mut hasher);
        hasher.finish()
    }
}

/// Information about a registered component
#[derive(Clone, Debug)]
pub struct ComponentInfo {
    /// The component's signature
    pub signature: ComponentSignature,
    /// Runtime component ID
    pub component_id: ComponentId,
    /// Hook state snapshot (serialized)
    pub hook_state: Option<String>,
    /// Props state snapshot (serialized)
    pub props_state: Option<String>,
    /// Timestamp when this component was last updated
    pub last_updated: u64,
    /// Number of times this component has been refreshed
    pub refresh_count: usize,
}

/// A pending component update
#[derive(Clone, Debug)]
pub struct ComponentUpdate {
    /// Old signature (before update)
    pub old_signature: ComponentSignature,
    /// New signature (after update)
    pub new_signature: ComponentSignature,
    /// Whether state should be preserved
    pub preserve_state: bool,
    /// Component ID to update
    pub component_id: ComponentId,
}

/// Result of a component registration
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RegistrationResult {
    /// Newly registered component
    Registered,
    /// Updated existing component
    Updated,
    /// Failed to preserve state
    StateLost,
}

/// Runtime for fast refresh state management
///
/// The FastRefreshRuntime tracks component signatures across hot reloads
/// and determines whether component state can be preserved.
pub struct FastRefreshRuntime {
    /// Registered components by signature
    registered_components: Arc<RwLock<HashMap<ComponentSignature, ComponentInfo>>>,
    /// Component IDs to signatures mapping
    component_signatures: Arc<RwLock<HashMap<ComponentId, ComponentSignature>>>,
    /// Pending updates to process
    pending_updates: Arc<Mutex<Vec<ComponentUpdate>>>,
    /// Components that should be fully re-rendered
    force_rerender: Arc<Mutex<HashSet<ComponentId>>>,
    /// Blacklisted components that cannot be fast refreshed
    blacklisted: Arc<RwLock<HashSet<ComponentSignature>>>,
}

impl Default for FastRefreshRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl FastRefreshRuntime {
    /// Create a new FastRefreshRuntime
    pub fn new() -> Self {
        Self {
            registered_components: Arc::new(RwLock::new(HashMap::new())),
            component_signatures: Arc::new(RwLock::new(HashMap::new())),
            pending_updates: Arc::new(Mutex::new(Vec::new())),
            force_rerender: Arc::new(Mutex::new(HashSet::new())),
            blacklisted: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Register a component with the runtime
    ///
    /// Returns whether the component was newly registered or updated.
    pub fn register_component(
        &self,
        signature: ComponentSignature,
        component_id: ComponentId,
    ) -> RegistrationResult {
        let mut components = self.registered_components.write().unwrap();
        let mut signatures = self.component_signatures.write().unwrap();

        // Check if this component ID was previously registered with a different signature
        let old_signature = signatures.get(&component_id);

        if let Some(old_sig) = old_signature {
            if old_sig != &signature {
                // Component signature changed - check if we can preserve state
                if self.can_preserve_state_internal(old_sig, &signature) {
                    // Update the component
                    let info = components.get_mut(old_sig).unwrap();
                    info.signature = signature.clone();
                    info.refresh_count += 1;

                    // Move to new signature key
                    let info = components.remove(old_sig).unwrap();
                    components.insert(signature.clone(), info);
                    signatures.insert(component_id, signature.clone());

                    RegistrationResult::Updated
                } else {
                    // Cannot preserve state - blacklist this component
                    let mut blacklist = self.blacklisted.write().unwrap();
                    blacklist.insert(signature.clone());
                    blacklist.insert(old_sig.clone());

                    RegistrationResult::StateLost
                }
            } else {
                RegistrationResult::Updated
            }
        } else {
            // New component registration
            let info = ComponentInfo {
                signature: signature.clone(),
                component_id,
                hook_state: None,
                props_state: None,
                last_updated: self.timestamp(),
                refresh_count: 0,
            };
            components.insert(signature.clone(), info);
            signatures.insert(component_id, signature);
            RegistrationResult::Registered
        }
    }

    /// Unregister a component from the runtime
    pub fn unregister_component(&self, component_id: ComponentId) -> bool {
        let mut signatures = self.component_signatures.write().unwrap();
        let mut components = self.registered_components.write().unwrap();

        if let Some(signature) = signatures.remove(&component_id) {
            components.remove(&signature);
            true
        } else {
            false
        }
    }

    /// Check if state can be preserved between two component versions
    pub fn can_preserve_state(
        &self,
        old_sig: &ComponentSignature,
        new_sig: &ComponentSignature,
    ) -> bool {
        self.can_preserve_state_internal(old_sig, new_sig)
    }

    fn can_preserve_state_internal(
        &self,
        old_sig: &ComponentSignature,
        new_sig: &ComponentSignature,
    ) -> bool {
        // Check blacklist
        let blacklist = self.blacklisted.read().unwrap();
        if blacklist.contains(old_sig) || blacklist.contains(new_sig) {
            return false;
        }

        // State can be preserved if:
        // 1. Component name hasn't changed
        // 2. File hasn't changed (component not moved)
        // 3. Hooks order hasn't changed (we can't detect this here, but we preserve the invariant)

        old_sig.name == new_sig.name && old_sig.file == new_sig.file
    }

    /// Queue a component update for processing
    pub fn queue_update(&self, update: ComponentUpdate) {
        let mut updates = self.pending_updates.lock().unwrap();
        updates.push(update);
    }

    /// Process all pending updates
    pub fn process_updates(&self) -> Vec<ComponentUpdate> {
        let mut updates = self.pending_updates.lock().unwrap();
        std::mem::take(&mut *updates)
    }

    /// Force a component to fully re-render on next update
    pub fn force_rerender(&self, component_id: ComponentId) {
        let mut force = self.force_rerender.lock().unwrap();
        force.insert(component_id);
    }

    /// Check if a component should force re-render
    pub fn should_force_rerender(&self, component_id: ComponentId) -> bool {
        let force = self.force_rerender.lock().unwrap();
        force.contains(&component_id)
    }

    /// Clear force re-render flag for a component
    pub fn clear_force_rerender(&self, component_id: ComponentId) {
        let mut force = self.force_rerender.lock().unwrap();
        force.remove(&component_id);
    }

    /// Get component info by signature
    pub fn get_component_info(&self, signature: &ComponentSignature) -> Option<ComponentInfo> {
        let components = self.registered_components.read().unwrap();
        components.get(signature).cloned()
    }

    /// Get component signature by ID
    pub fn get_component_signature(&self, component_id: ComponentId) -> Option<ComponentSignature> {
        let signatures = self.component_signatures.read().unwrap();
        signatures.get(&component_id).cloned()
    }

    /// Get all registered components
    pub fn get_all_components(&self) -> Vec<ComponentInfo> {
        let components = self.registered_components.read().unwrap();
        components.values().cloned().collect()
    }

    /// Clear all registered components (useful for full reload)
    pub fn clear(&self) {
        let mut components = self.registered_components.write().unwrap();
        let mut signatures = self.component_signatures.write().unwrap();
        let mut force = self.force_rerender.lock().unwrap();

        components.clear();
        signatures.clear();
        force.clear();
    }

    /// Add a signature to the blacklist
    pub fn blacklist(&self, signature: ComponentSignature) {
        let mut blacklist = self.blacklisted.write().unwrap();
        blacklist.insert(signature);
    }

    /// Check if a signature is blacklisted
    pub fn is_blacklisted(&self, signature: &ComponentSignature) -> bool {
        let blacklist = self.blacklisted.read().unwrap();
        blacklist.contains(signature)
    }

    /// Get current timestamp in milliseconds
    fn timestamp(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Sign a component for fast refresh
///
/// This creates a signature that identifies the component across hot reloads.
/// Call this at the component's declaration site.
///
/// # Example
///
/// ```
/// # use tairitsu_ssr::fast_refresh::sign_component;
/// let sig = sign_component("MyComponent", "src/app.rs", 42);
/// ```
pub fn sign_component(name: &str, file: &str, line: u32) -> ComponentSignature {
    ComponentSignature::new(name, file, line)
}

/// Check if state can be preserved between updates
///
/// This is a convenience function that creates a runtime and checks compatibility.
pub fn can_preserve_state(old_sig: &ComponentSignature, new_sig: &ComponentSignature) -> bool {
    let runtime = FastRefreshRuntime::new();
    runtime.can_preserve_state(old_sig, new_sig)
}

/// Macro to easily create a component signature at compile time
///
/// # Example
///
/// ```
/// # use tairitsu_ssr::sign_component_macro;
/// let sig = sign_component_macro!("MyComponent");
/// ```
#[macro_export]
macro_rules! sign_component_macro {
    ($name:expr) => {
        $crate::sign_component($name, file!(), line!())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_signature_creation() {
        let sig = ComponentSignature::new("TestComponent", "src/test.rs", 42);
        assert_eq!(sig.name, "TestComponent");
        assert_eq!(sig.file, "src/test.rs");
        assert_eq!(sig.line, 42);
    }

    #[test]
    fn test_component_signature_equality() {
        let sig1 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig2 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig3 = ComponentSignature::new("Test", "src/test.rs", 11);

        assert_eq!(sig1, sig2);
        assert_ne!(sig1, sig3);
    }

    #[test]
    fn test_component_signature_hash() {
        let sig1 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig2 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig3 = ComponentSignature::new("Test", "src/other.rs", 10);

        assert_eq!(sig1.hash(), sig2.hash());
        assert_ne!(sig1.hash(), sig3.hash());
    }

    #[test]
    fn test_component_signature_parse() {
        let sig1 = ComponentSignature::parse("function MyComponent", "src/app.rs");
        assert!(sig1.is_some());
        assert_eq!(sig1.unwrap().name, "MyComponent");

        let sig2 = ComponentSignature::parse("const Button = ()", "src/button.rs");
        assert!(sig2.is_some());
        assert_eq!(sig2.unwrap().name, "Button");

        let sig3 = ComponentSignature::parse("invalid syntax", "src/test.rs");
        assert!(sig3.is_none());
    }

    #[test]
    fn test_runtime_register_component() {
        let runtime = FastRefreshRuntime::new();
        let sig = ComponentSignature::new("Test", "src/test.rs", 10);

        let result = runtime.register_component(sig.clone(), 1);
        assert_eq!(result, RegistrationResult::Registered);

        let info = runtime.get_component_info(&sig).unwrap();
        assert_eq!(info.component_id, 1);
        assert_eq!(info.refresh_count, 0);
    }

    #[test]
    fn test_runtime_update_component() {
        let runtime = FastRefreshRuntime::new();
        let sig1 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig2 = ComponentSignature::new("Test", "src/test.rs", 20);

        runtime.register_component(sig1.clone(), 1);
        let result = runtime.register_component(sig2.clone(), 1);

        assert_eq!(result, RegistrationResult::Updated);

        let info = runtime.get_component_info(&sig2).unwrap();
        assert_eq!(info.component_id, 1);
        assert_eq!(info.refresh_count, 1);
    }

    #[test]
    fn test_runtime_state_preservation() {
        let runtime = FastRefreshRuntime::new();
        let sig1 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig2 = ComponentSignature::new("Test", "src/test.rs", 20);

        assert!(runtime.can_preserve_state(&sig1, &sig2));

        let sig3 = ComponentSignature::new("Other", "src/test.rs", 10);
        assert!(!runtime.can_preserve_state(&sig1, &sig3));

        let sig4 = ComponentSignature::new("Test", "src/other.rs", 10);
        assert!(!runtime.can_preserve_state(&sig1, &sig4));
    }

    #[test]
    fn test_runtime_unregister_component() {
        let runtime = FastRefreshRuntime::new();
        let sig = ComponentSignature::new("Test", "src/test.rs", 10);

        runtime.register_component(sig.clone(), 1);
        assert!(runtime.unregister_component(1));
        assert!(!runtime.unregister_component(1));

        assert!(runtime.get_component_info(&sig).is_none());
    }

    #[test]
    fn test_runtime_force_rerender() {
        let runtime = FastRefreshRuntime::new();

        assert!(!runtime.should_force_rerender(1));
        runtime.force_rerender(1);
        assert!(runtime.should_force_rerender(1));
        runtime.clear_force_rerender(1);
        assert!(!runtime.should_force_rerender(1));
    }

    #[test]
    fn test_runtime_blacklist() {
        let runtime = FastRefreshRuntime::new();
        let sig = ComponentSignature::new("Test", "src/test.rs", 10);

        assert!(!runtime.is_blacklisted(&sig));
        runtime.blacklist(sig.clone());
        assert!(runtime.is_blacklisted(&sig));
    }

    #[test]
    fn test_runtime_queue_and_process_updates() {
        let runtime = FastRefreshRuntime::new();
        let sig1 = ComponentSignature::new("Test", "src/test.rs", 10);
        let sig2 = ComponentSignature::new("Test", "src/test.rs", 20);

        let update = ComponentUpdate {
            old_signature: sig1,
            new_signature: sig2,
            preserve_state: true,
            component_id: 1,
        };

        runtime.queue_update(update.clone());
        let updates = runtime.process_updates();

        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].component_id, 1);
    }

    #[test]
    fn test_runtime_clear() {
        let runtime = FastRefreshRuntime::new();
        let sig = ComponentSignature::new("Test", "src/test.rs", 10);

        runtime.register_component(sig.clone(), 1);
        runtime.force_rerender(1);
        runtime.clear();

        assert!(runtime.get_component_info(&sig).is_none());
        assert!(!runtime.should_force_rerender(1));
    }

    #[test]
    fn test_sign_component() {
        let sig = sign_component("MyComponent", "src/app.rs", 42);
        assert_eq!(sig.name, "MyComponent");
        assert_eq!(sig.file, "src/app.rs");
        assert_eq!(sig.line, 42);
    }

    #[test]
    fn test_component_info_serialization() {
        let sig = ComponentSignature::new("Test", "src/test.rs", 10);
        let json = serde_json::to_string(&sig).unwrap();
        let decoded: ComponentSignature = serde_json::from_str(&json).unwrap();

        assert_eq!(sig, decoded);
    }
}
