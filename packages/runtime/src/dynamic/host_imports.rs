//! Host Import Registry for dynamic host function invocation
//!
//! This module provides a registry for host functions that WASM components import,
//! allowing dynamic invocation similar to guest exports.

use std::{collections::HashMap, sync::Arc};
use anyhow::Result;

use wasmtime::component::{Val, Type};

/// Host import function registry
pub struct HostImportRegistry {
    imports: HashMap<String, HostImport>,
}

impl HostImportRegistry {
    pub fn new() -> Self {
        Self {
            imports: HashMap::new(),
        }
    }

    /// Register a host import function
    pub fn register<F>(
        &mut self,
        name: String,
        params: Vec<Type>,
        results: Vec<Type>,
        handler: F,
    ) where
        F: Fn(&[Val]) -> Result<Vec<Val>> + Send + Sync + 'static,
    {
        let import = HostImport {
            name: name.clone(),
            params,
            results,
            handler: Arc::new(handler),
        };
        self.imports.insert(name, import);
    }

    /// Dynamically call a host import function
    pub fn call(&self, name: &str, args: &[Val]) -> Result<Vec<Val>> {
        let import = self
            .imports
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Host import not found: {}", name))?;

        (import.handler)(args)
    }

    /// List all registered import functions
    pub fn list_imports(&self) -> Vec<&str> {
        self.imports.keys().map(|k| k.as_str()).collect()
    }

    /// Get function signature
    pub fn get_signature(&self, name: &str) -> Option<(Vec<Type>, Vec<Type>)> {
        self.imports.get(name).map(|i| (i.params.clone(), i.results.clone()))
    }
}

impl Default for HostImportRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Host import function descriptor
pub struct HostImport {
    name: String,
    params: Vec<Type>,
    results: Vec<Type>,
    handler: Arc<dyn Fn(&[Val]) -> Result<Vec<Val>> + Send + Sync>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_host_import_registry() {
        let mut registry = HostImportRegistry::new();

        // Register a simple function
        registry.register(
            "test-func".to_string(),
            vec![],
            vec![],
            |_args| Ok(vec![]),
        );

        assert!(registry.list_imports().contains(&"test-func"));
    }

    #[test]
    fn test_host_import_call() {
        let mut registry = HostImportRegistry::new();

        registry.register(
            "add".to_string(),
            vec![Type::U32, Type::U32],
            vec![Type::U32],
            |args| {
                let a = match &args[0] {
                    Val::U32(n) => *n,
                    _ => return Err(anyhow::anyhow!("Type error")),
                };
                let b = match &args[1] {
                    Val::U32(n) => *n,
                    _ => return Err(anyhow::anyhow!("Type error")),
                };
                Ok(vec![Val::U32(a + b)])
            },
        );

        let result = registry.call("add", &[Val::U32(10), Val::U32(32)]).unwrap();
        assert_eq!(result[0], Val::U32(42));
    }
}
