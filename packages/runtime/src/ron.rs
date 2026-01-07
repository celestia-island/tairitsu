//! RON serialization layer for dynamic WASM invocation
//!
//! Similar to JsonBinding but using RON for better Rust type compatibility.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

/// RON binding utilities for WIT types
pub struct RonBinding;

impl RonBinding {
    /// Convert parameters to RON string
    pub fn params_to_ron<T: Serialize>(params: &T) -> Result<String> {
        ron::to_string(params).map_err(Into::into)
    }

    /// Convert RON string back to parameters
    pub fn ron_to_params<'de, T: Deserialize<'de>>(ron: &'de str) -> Result<T> {
        ron::from_str(ron).map_err(Into::into)
    }

    /// Convert parameters to RON bytes
    pub fn params_to_ron_bytes<T: Serialize>(params: &T) -> Result<Vec<u8>> {
        // RON doesn't have direct to_vec, so we serialize to string then convert to bytes
        let ron_str = ron::to_string(params)?;
        Ok(ron_str.into_bytes())
    }

    /// Convert RON bytes back to parameters
    pub fn ron_bytes_to_params<'de, T: Deserialize<'de>>(ron: &'de [u8]) -> Result<T> {
        // Convert bytes to string, then parse as RON
        let ron_str = std::str::from_utf8(ron).context("RON bytes are not valid UTF-8")?;
        ron::from_str(ron_str).map_err(Into::into)
    }
}

/// Dynamic tool/function registry for RON-based invocation
///
/// Similar to ToolRegistry but uses RON for serialization.
pub struct RonToolRegistry {
    tools: HashMap<String, Arc<dyn RonTool>>,
}

impl RonToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: String, tool: Arc<dyn RonTool>) {
        self.tools.insert(name, tool);
    }

    pub fn invoke(&self, name: &str, ron: &str) -> Result<String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        tool.invoke_ron(ron)
    }

    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for RonToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for dynamic tool invocation using RON
pub trait RonTool: Send + Sync {
    fn invoke_ron(&self, ron: &str) -> Result<String>;
    fn name(&self) -> &str;
}

/// Helper to create a typed tool with RON serialization
pub fn typed_ron_tool<I, O, F>(name: &str, f: F) -> Arc<dyn RonTool>
where
    I: for<'de> Deserialize<'de> + Send + 'static,
    O: Serialize + Send + 'static,
    F: Fn(I) -> O + Send + Sync + 'static,
{
    let name = name.to_string();
    Arc::new(RonFunctionTool::new(name.clone(), move |ron| {
        let input: I = ron::from_str(ron)?;
        let output = f(input);
        Ok(ron::to_string(&output)?)
    }))
}

/// Simple function-based tool using RON
pub struct RonFunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    name: String,
    func: F,
}

impl<F> RonFunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

impl<F> RonTool for RonFunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    fn invoke_ron(&self, ron: &str) -> Result<String> {
        (self.func)(ron)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ron_binding() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestParams {
            message: String,
            count: u32,
        }

        let params = TestParams {
            message: "hello".to_string(),
            count: 42,
        };

        let ron = RonBinding::params_to_ron(&params).unwrap();
        let decoded: TestParams = RonBinding::ron_to_params(&ron).unwrap();

        assert_eq!(params, decoded);
    }

    #[test]
    fn test_ron_tool_registry() {
        let tool = typed_ron_tool("echo", |input: String| -> String {
            format!("echo: {}", input)
        });

        let mut registry = RonToolRegistry::new();
        registry.register("echo".to_string(), tool);

        assert!(registry.has_tool("echo"));
        assert_eq!(registry.list_tools(), vec!["echo"]);

        // RON format for strings: "hello" (with double quotes)
        let result = registry.invoke("echo", r#""hello""#).unwrap();
        // Result should be a RON-encoded string
        assert!(result.contains("echo: hello"));
    }

    #[test]
    fn test_typed_ron_tool() {
        #[derive(Deserialize)]
        struct AddInput {
            a: i32,
            b: i32,
        }

        #[derive(Serialize)]
        struct AddOutput {
            result: i32,
        }

        let tool = typed_ron_tool("add", |input: AddInput| -> AddOutput {
            AddOutput {
                result: input.a + input.b,
            }
        });

        let mut registry = RonToolRegistry::new();
        registry.register("add".to_string(), tool);

        // RON format for structs
        let result = registry.invoke("add", r#"(a: 10, b: 32)"#).unwrap();
        assert!(result.contains("42"));
    }
}
