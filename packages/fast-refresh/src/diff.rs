//! Component diffing for Fast Refresh
//!
//! This module provides utilities for comparing component definitions
//! to determine if state can be preserved during hot reload.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::ComponentSignature;

/// Result of diffing two components
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffResult {
    /// Components are identical - state can be preserved
    Identical,
    /// Components are compatible - state can be preserved
    Compatible { changes: Vec<ComponentChange> },
    /// Components are incompatible - state will be lost
    Incompatible { reason: String },
}

/// Types of changes detected between component versions
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentChange {
    /// Hooks changed but order is preserved
    HooksChanged(HookChange),
    /// Function body changed
    FunctionChanged(FunctionChange),
    /// Properties/props interface changed
    PropertiesChanged(PropertyChange),
    /// Added new imports or dependencies
    ImportsChanged { added: Vec<String>, removed: Vec<String> },
}

/// Hook-related changes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookChange {
    /// Hook added at the end (safe)
    HookAdded { name: String, position: usize },
    /// Hook removed (might be safe if at end)
    HookRemoved { name: String, position: usize },
    /// Hook order changed (unsafe)
    HookOrderChanged { hook: String, old_pos: usize, new_pos: usize },
    /// Hook type changed (e.g., useState -> useReducer)
    HookTypeChanged { position: usize, old_type: String, new_type: String },
}

/// Function-related changes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionChange {
    /// Function body changed but signature is the same
    BodyChanged,
    /// Function signature changed (unsafe)
    SignatureChanged { old_params: usize, new_params: usize },
    /// Function became async or sync changed
    AsyncChanged { became_async: bool },
}

/// Property-related changes
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyChange {
    /// Property added (might be safe with default)
    PropertyAdded { name: String, has_default: bool },
    /// Property removed (unsafe)
    PropertyRemoved { name: String },
    /// Property type changed (unsafe)
    PropertyTypeChanged { name: String, old_type: String, new_type: String },
    /// Property renamed (unsafe)
    PropertyRenamed { old_name: String, new_name: String },
}

/// Metadata about a component for diffing
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Component signature
    pub signature: ComponentSignature,
    /// Hooks used by the component in order
    pub hooks: Vec<HookInfo>,
    /// Properties/props accepted by the component
    pub properties: Vec<PropertyInfo>,
    /// Imports used by the component
    pub imports: Vec<String>,
    /// Whether the function is async
    pub is_async: bool,
    /// Number of function parameters
    pub param_count: usize,
}

/// Information about a hook used in a component
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookInfo {
    /// Hook name (e.g., "useState", "useEffect")
    pub name: String,
    /// Hook type/category (state, effect, callback, etc.)
    pub hook_type: HookType,
    /// Position in the component (0-indexed)
    pub position: usize,
}

/// Types of hooks
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    /// State hook (useState, useReducer)
    State,
    /// Effect hook (useEffect, useLayoutEffect)
    Effect,
    /// Callback hook (useCallback, useMemo)
    Callback,
    /// Ref hook (useRef, useRefObject)
    Ref,
    /// Context hook (useContext)
    Context,
    /// Custom hook
    Custom(String),
    /// Unknown hook
    Unknown,
}

/// Information about a component property
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyInfo {
    /// Property name
    pub name: String,
    /// Property type
    pub type_name: String,
    /// Whether the property has a default value
    pub has_default: bool,
    /// Whether the property is required
    pub required: bool,
}

/// Diff two component metadata to determine compatibility
///
/// # Example
///
/// ```
/// # use tairitsu_fast_refresh::{ComponentSignature, ComponentMetadata, diff_components};
/// # use tairitsu_fast_refresh::HookType;
/// let sig = ComponentSignature::new("Test", "src/test.rs", 10);
/// let old_meta = ComponentMetadata::new(sig.clone());
/// let new_meta = ComponentMetadata::new(sig);
///
/// let result = diff_components(&old_meta, &new_meta);
/// ```
pub fn diff_components(old: &ComponentMetadata, new: &ComponentMetadata) -> DiffResult {
    // Check if signatures are compatible (name and file must match)
    if old.signature.name != new.signature.name {
        return DiffResult::Incompatible {
            reason: format!(
                "Component name changed: {} -> {}",
                old.signature.name, new.signature.name
            ),
        };
    }

    if old.signature.file != new.signature.file {
        return DiffResult::Incompatible {
            reason: format!(
                "Component file changed: {} -> {}",
                old.signature.file, new.signature.file
            ),
        };
    }

    let mut changes = Vec::new();
    let mut has_incompatible_change = false;

    // Check for hook changes
    if let Some(hook_change) = diff_hooks(&old.hooks, &new.hooks) {
        if is_hook_change_incompatible(&hook_change) {
            has_incompatible_change = true;
        }
        changes.push(ComponentChange::HooksChanged(hook_change));
    }

    // Check for function changes
    if let Some(func_change) = diff_function(old, new) {
        if is_function_change_incompatible(&func_change) {
            has_incompatible_change = true;
        }
        changes.push(ComponentChange::FunctionChanged(func_change));
    }

    // Check for property changes
    if let Some(prop_change) = diff_properties(&old.properties, &new.properties) {
        if is_property_change_incompatible(&prop_change) {
            has_incompatible_change = true;
        }
        changes.push(ComponentChange::PropertiesChanged(prop_change));
    }

    // Check for import changes (generally compatible)
    let (added_imports, removed_imports) = diff_imports(&old.imports, &new.imports);
    if !added_imports.is_empty() || !removed_imports.is_empty() {
        changes.push(ComponentChange::ImportsChanged {
            added: added_imports,
            removed: removed_imports,
        });
    }

    if has_incompatible_change {
        DiffResult::Incompatible {
            reason: "Component has incompatible changes that prevent state preservation".to_string(),
        }
    } else if changes.is_empty() {
        DiffResult::Identical
    } else {
        DiffResult::Compatible { changes }
    }
}

/// Diff hook arrays between component versions
fn diff_hooks(old: &[HookInfo], new: &[HookInfo]) -> Option<HookChange> {
    if old == new {
        return None;
    }

    // Check for hook order changes
    for (i, (old_hook, new_hook)) in old.iter().zip(new.iter()).enumerate() {
        if old_hook.name != new_hook.name {
            return Some(HookChange::HookOrderChanged {
                hook: old_hook.name.clone(),
                old_pos: i,
                new_pos: i,
            });
        }
        if old_hook.hook_type != new_hook.hook_type {
            return Some(HookChange::HookTypeChanged {
                position: i,
                old_type: format!("{:?}", old_hook.hook_type),
                new_type: format!("{:?}", new_hook.hook_type),
            });
        }
    }

    // Check for added hooks
    if new.len() > old.len() {
        return Some(HookChange::HookAdded {
            name: new[old.len()].name.clone(),
            position: old.len(),
        });
    }

    // Check for removed hooks
    if new.len() < old.len() {
        return Some(HookChange::HookRemoved {
            name: old[new.len()].name.clone(),
            position: new.len(),
        });
    }

    None
}

/// Check if a hook change is incompatible with state preservation
fn is_hook_change_incompatible(change: &HookChange) -> bool {
    match change {
        HookChange::HookOrderChanged { .. } => true,
        HookChange::HookTypeChanged { .. } => true,
        HookChange::HookRemoved { position, .. } => {
            // Removing hooks from the middle is unsafe
            // Removing from the end might be safe in some cases
            *position > 0
        }
        HookChange::HookAdded { .. } => false,
    }
}

/// Diff function metadata between component versions
fn diff_function(old: &ComponentMetadata, new: &ComponentMetadata) -> Option<FunctionChange> {
    // If we're comparing empty metadata, assume no function change
    // (this is the case when both components are newly created)
    if old.hooks.is_empty() && new.hooks.is_empty()
        && old.properties.is_empty() && new.properties.is_empty()
        && old.imports.is_empty() && new.imports.is_empty() {
        return None;
    }

    if old.is_async != new.is_async {
        return Some(FunctionChange::AsyncChanged {
            became_async: new.is_async,
        });
    }

    if old.param_count != new.param_count {
        return Some(FunctionChange::SignatureChanged {
            old_params: old.param_count,
            new_params: new.param_count,
        });
    }

    // If we have actual content (hooks, properties, imports) and they differ,
    // assume the function body changed
    if !old.hooks.is_empty() || !new.hooks.is_empty()
        || !old.properties.is_empty() || !new.properties.is_empty()
        || !old.imports.is_empty() || !new.imports.is_empty() {
        Some(FunctionChange::BodyChanged)
    } else {
        None
    }
}

/// Check if a function change is incompatible with state preservation
fn is_function_change_incompatible(change: &FunctionChange) -> bool {
    match change {
        FunctionChange::BodyChanged => false,
        FunctionChange::SignatureChanged { .. } => true,
        FunctionChange::AsyncChanged { .. } => true,
    }
}

/// Diff property arrays between component versions
fn diff_properties(old: &[PropertyInfo], new: &[PropertyInfo]) -> Option<PropertyChange> {
    let old_props: HashSet<_> = old.iter().map(|p| &p.name).collect();
    let new_props: HashSet<_> = new.iter().map(|p| &p.name).collect();

    // Check for removed properties
    for prop in old {
        if !new_props.contains(&prop.name) {
            return Some(PropertyChange::PropertyRemoved {
                name: prop.name.clone(),
            });
        }
    }

    // Check for added properties
    for prop in new {
        if !old_props.contains(&prop.name) {
            return Some(PropertyChange::PropertyAdded {
                name: prop.name.clone(),
                has_default: prop.has_default,
            });
        }
    }

    // Check for type changes
    for new_prop in new {
        if let Some(old_prop) = old.iter().find(|p| p.name == new_prop.name)
            && old_prop.type_name != new_prop.type_name {
                return Some(PropertyChange::PropertyTypeChanged {
                    name: new_prop.name.clone(),
                    old_type: old_prop.type_name.clone(),
                    new_type: new_prop.type_name.clone(),
                });
            }
    }

    None
}

/// Check if a property change is incompatible with state preservation
fn is_property_change_incompatible(change: &PropertyChange) -> bool {
    match change {
        PropertyChange::PropertyAdded { has_default, .. } => !has_default,
        PropertyChange::PropertyRemoved { .. } => true,
        PropertyChange::PropertyTypeChanged { .. } => true,
        PropertyChange::PropertyRenamed { .. } => true,
    }
}

/// Diff import arrays between component versions
fn diff_imports(old: &[String], new: &[String]) -> (Vec<String>, Vec<String>) {
    let old_set: HashSet<_> = old.iter().cloned().collect();
    let new_set: HashSet<_> = new.iter().cloned().collect();

    let added: Vec<_> = new_set.difference(&old_set).cloned().collect();
    let removed: Vec<_> = old_set.difference(&new_set).cloned().collect();

    (added, removed)
}

impl ComponentMetadata {
    /// Create empty metadata for a component
    pub fn new(signature: ComponentSignature) -> Self {
        Self {
            signature,
            hooks: Vec::new(),
            properties: Vec::new(),
            imports: Vec::new(),
            is_async: false,
            param_count: 0,
        }
    }

    /// Add a hook to the metadata
    pub fn with_hook(mut self, name: impl Into<String>, hook_type: HookType) -> Self {
        let position = self.hooks.len();
        self.hooks.push(HookInfo {
            name: name.into(),
            hook_type,
            position,
        });
        self
    }

    /// Add a property to the metadata
    pub fn with_property(
        mut self,
        name: impl Into<String>,
        type_name: impl Into<String>,
        has_default: bool,
        required: bool,
    ) -> Self {
        self.properties.push(PropertyInfo {
            name: name.into(),
            type_name: type_name.into(),
            has_default,
            required,
        });
        self
    }

    /// Add an import to the metadata
    pub fn with_import(mut self, import: impl Into<String>) -> Self {
        self.imports.push(import.into());
        self
    }

    /// Set whether the component is async
    pub fn with_async(mut self, is_async: bool) -> Self {
        self.is_async = is_async;
        self
    }

    /// Set the parameter count
    pub fn with_params(mut self, count: usize) -> Self {
        self.param_count = count;
        self
    }
}

impl HookInfo {
    /// Create a new hook info
    pub fn new(name: impl Into<String>, hook_type: HookType, position: usize) -> Self {
        Self {
            name: name.into(),
            hook_type,
            position,
        }
    }

    /// Detect hook type from name
    pub fn detect_type(name: &str) -> HookType {
        match name {
            "useState" | "useReducer" | "useStateMachine" => HookType::State,
            "useEffect" | "useLayoutEffect" | "useInsertionEffect" => HookType::Effect,
            "useCallback" | "useMemo" => HookType::Callback,
            "useRef" | "useRefObject" => HookType::Ref,
            "useContext" => HookType::Context,
            _ if name.starts_with("use") => HookType::Custom(name[3..].to_string()),
            _ => HookType::Unknown,
        }
    }

    /// Create hook info with auto-detected type
    pub fn with_auto_type(name: impl Into<String>, position: usize) -> Self {
        let name = name.into();
        let hook_type = Self::detect_type(&name);
        Self::new(name, hook_type, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ComponentSignature;

    fn make_signature(name: &str) -> ComponentSignature {
        ComponentSignature::new(name, "src/test.rs", 10)
    }

    #[test]
    fn test_diff_identical_components() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone());
        let meta2 = ComponentMetadata::new(sig);

        let result = diff_components(&meta1, &meta2);
        assert_eq!(result, DiffResult::Identical);
    }

    #[test]
    fn test_diff_incompatible_name_change() {
        let sig1 = make_signature("Test");
        let sig2 = ComponentSignature::new("Other", "src/test.rs", 10);
        let meta1 = ComponentMetadata::new(sig1);
        let meta2 = ComponentMetadata::new(sig2);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_hook_added() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone());
        let meta2 = meta1.clone().with_hook("useState", HookType::State);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Compatible { .. }));
    }

    #[test]
    fn test_diff_hook_order_changed() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone())
            .with_hook("useState", HookType::State)
            .with_hook("useEffect", HookType::Effect);
        let meta2 = ComponentMetadata::new(sig)
            .with_hook("useEffect", HookType::Effect)
            .with_hook("useState", HookType::State);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_hook_type_changed() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone())
            .with_hook("useState", HookType::State);
        let meta2 = ComponentMetadata::new(sig)
            .with_hook("useRef", HookType::Ref);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_property_added_with_default() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone());
        let meta2 = meta1
            .clone()
            .with_property("label", "string", true, false);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Compatible { .. }));
    }

    #[test]
    fn test_diff_property_added_without_default() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone());
        let meta2 = meta1
            .clone()
            .with_property("label", "string", false, true);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_property_removed() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone())
            .with_property("label", "string", true, false);
        let meta2 = ComponentMetadata::new(sig);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_property_type_changed() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone())
            .with_property("count", "number", true, false);
        let meta2 = ComponentMetadata::new(sig)
            .with_property("count", "string", true, false);

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Incompatible { .. }));
    }

    #[test]
    fn test_diff_imports_changed() {
        let sig = make_signature("Test");
        let meta1 = ComponentMetadata::new(sig.clone()).with_import("react");
        let meta2 = meta1
            .clone()
            .with_import("react")
            .with_import("lodash");

        let result = diff_components(&meta1, &meta2);
        assert!(matches!(result, DiffResult::Compatible { .. }));

        if let DiffResult::Compatible { changes } = result {
            assert!(changes.iter().any(|c| matches!(
                c,
                ComponentChange::ImportsChanged { added, .. } if added.contains(&"lodash".to_string())
            )));
        }
    }

    #[test]
    fn test_hook_info_detect_type() {
        assert_eq!(
            HookInfo::detect_type("useState"),
            HookType::State
        );
        assert_eq!(
            HookInfo::detect_type("useEffect"),
            HookType::Effect
        );
        assert_eq!(
            HookInfo::detect_type("useCallback"),
            HookType::Callback
        );
        assert_eq!(
            HookInfo::detect_type("useRef"),
            HookType::Ref
        );
        assert_eq!(
            HookInfo::detect_type("useContext"),
            HookType::Context
        );
        assert_eq!(
            HookInfo::detect_type("useCustomHook"),
            HookType::Custom("CustomHook".to_string())
        );
        assert_eq!(HookInfo::detect_type("regularFunction"), HookType::Unknown);
    }

    #[test]
    fn test_hook_info_with_auto_type() {
        let info = HookInfo::with_auto_type("useState", 0);
        assert_eq!(info.name, "useState");
        assert_eq!(info.hook_type, HookType::State);
        assert_eq!(info.position, 0);
    }

    #[test]
    fn test_component_metadata_builder() {
        let sig = make_signature("Test");
        let meta = ComponentMetadata::new(sig.clone())
            .with_hook("useState", HookType::State)
            .with_property("count", "number", true, false)
            .with_import("react")
            .with_async(false)
            .with_params(1);

        assert_eq!(meta.signature, sig);
        assert_eq!(meta.hooks.len(), 1);
        assert_eq!(meta.properties.len(), 1);
        assert_eq!(meta.imports.len(), 1);
        assert!(!meta.is_async);
        assert_eq!(meta.param_count, 1);
    }

    #[test]
    fn test_metadata_serialization() {
        let sig = make_signature("Test");
        let meta = ComponentMetadata::new(sig);
        let json = serde_json::to_string(&meta).unwrap();
        let decoded: ComponentMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta, decoded);
    }
}
