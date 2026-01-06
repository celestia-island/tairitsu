//! JSON serialization layer for WIT interfaces
//!
//! This module provides JSON-based serialization/deserialization support
//! for dynamic invocation scenarios where you need to call WIT functions
//! with JSON payloads.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// JSON binding utilities for WIT types
pub struct JsonBinding;

impl JsonBinding {
    /// Convert parameters to JSON string
    ///
    /// # Arguments
    /// * `params` - Any type that implements Serialize
    ///
    /// # Example
    /// ```
    /// use tairitsu::json::JsonBinding;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct MyParams {
    ///     message: String,
    ///     count: u32,
    /// }
    ///
    /// let params = MyParams {
    ///     message: "hello".to_string(),
    ///     count: 42,
    /// };
    ///
    /// let json = JsonBinding::params_to_json(&params).unwrap();
    /// assert!(json.contains("hello"));
    /// ```
    pub fn params_to_json<T: Serialize>(params: &T) -> Result<String> {
        serde_json::to_string_pretty(params).map_err(Into::into)
    }

    /// Convert JSON string back to parameters
    ///
    /// # Arguments
    /// * `json` - JSON string
    ///
    /// # Type Parameters
    /// * `T` - Target type that implements Deserialize
    ///
    /// # Example
    /// ```
    /// use tairitsu::json::JsonBinding;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize, Debug, PartialEq)]
    /// struct MyParams {
    ///     message: String,
    ///     count: u32,
    /// }
    ///
    /// let json = r#"{"message":"hello","count":42}"#;
    /// let params: MyParams = JsonBinding::json_to_params(json).unwrap();
    /// assert_eq!(params.message, "hello");
    /// assert_eq!(params.count, 42);
    /// ```
    pub fn json_to_params<'de, T: Deserialize<'de>>(json: &'de str) -> Result<T> {
        serde_json::from_str(json).map_err(Into::into)
    }

    /// Convert parameters to JSON bytes
    pub fn params_to_json_bytes<T: Serialize>(params: &T) -> Result<Vec<u8>> {
        serde_json::to_vec(params).map_err(Into::into)
    }

    /// Convert JSON bytes back to parameters
    pub fn json_bytes_to_params<'de, T: Deserialize<'de>>(json: &'de [u8]) -> Result<T> {
        serde_json::from_slice(json).map_err(Into::into)
    }
}

/// Dynamic tool/function registry for JSON-based invocation
///
/// This allows you to register functions that can be called dynamically
/// with JSON payloads, useful for RPC-style APIs or plugin systems.
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Register a tool
    ///
    /// # Arguments
    /// * `name` - Unique name for the tool
    /// * `tool` - Tool implementation
    ///
    /// # Example
    /// ```
    /// use tairitsu::json::{ToolRegistry, Tool};
    /// use anyhow::Result;
    /// use serde::{Deserialize, Serialize};
    /// use std::sync::Arc;
    ///
    /// struct MyTool;
    ///
    /// #[derive(Deserialize)]
    /// struct MyInput {
    ///     value: String,
    /// }
    ///
    /// #[derive(Serialize)]
    /// struct MyOutput {
    ///     result: String,
    /// }
    ///
    /// impl Tool for MyTool {
    ///     fn invoke_json(&self, json: &str) -> Result<String> {
    ///         let input: MyInput = serde_json::from_str(json)?;
    ///         let output = MyOutput {
    ///             result: format!("processed: {}", input.value),
    ///         };
    ///         Ok(serde_json::to_string(&output)?)
    ///     }
    ///
    ///     fn name(&self) -> &str {
    ///         "my-tool"
    ///     }
    /// }
    ///
    /// let mut registry = ToolRegistry::new();
    /// registry.register("my-tool".to_string(), Arc::new(MyTool));
    /// ```
    pub fn register(&mut self, name: String, tool: Arc<dyn Tool>) {
        self.tools.insert(name, tool);
    }

    /// Invoke a tool by name with JSON payload
    ///
    /// # Arguments
    /// * `name` - Tool name
    /// * `json` - JSON input string
    ///
    /// # Returns
    /// JSON output string
    ///
    /// # Errors
    /// Returns error if tool not found or invocation fails
    pub fn invoke(&self, name: &str, json: &str) -> Result<String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
        tool.invoke_json(json)
    }

    /// List all registered tool names
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|k| k.as_str()).collect()
    }

    /// Check if a tool is registered
    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.contains_key(name)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for dynamic tool invocation
///
/// Tools can be any function or operation that accepts JSON input
/// and produces JSON output.
pub trait Tool: Send + Sync {
    /// Invoke the tool with JSON input
    ///
    /// # Arguments
    /// * `json` - JSON input string
    ///
    /// # Returns
    /// JSON output string
    fn invoke_json(&self, json: &str) -> Result<String>;

    /// Get the tool's name
    fn name(&self) -> &str;
}

/// Simple function-based tool
///
/// Wraps a closure or function pointer as a Tool implementation.
pub struct FunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    name: String,
    func: F,
}

impl<F> FunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    /// Create a new function tool
    ///
    /// # Arguments
    /// * `name` - Tool name
    /// * `func` - Function that takes JSON input and returns JSON output
    pub fn new(name: String, func: F) -> Self {
        Self { name, func }
    }
}

impl<F> Tool for FunctionTool<F>
where
    F: Fn(&str) -> Result<String> + Send + Sync,
{
    fn invoke_json(&self, json: &str) -> Result<String> {
        (self.func)(json)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Helper to create a typed tool
///
/// This makes it easier to create tools with typed input/output.
///
/// # Example
/// /// ```
/// use tairitsu::json::{typed_tool, ToolRegistry};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct AddInput {
///     a: i32,
///     b: i32,
/// }
///
/// #[derive(Serialize)]
/// struct AddOutput {
///     result: i32,
/// }
///
/// let tool = typed_tool("add", |input: AddInput| -> AddOutput {
///     AddOutput { result: input.a + input.b }
/// });
///
/// let mut registry = ToolRegistry::new();
/// registry.register("add".to_string(), tool);
/// ```
pub fn typed_tool<I, O, F>(name: &str, f: F) -> Arc<dyn Tool>
where
    I: for<'de> Deserialize<'de> + Send + 'static,
    O: Serialize + Send + 'static,
    F: Fn(I) -> O + Send + Sync + 'static,
{
    let name = name.to_string();
    Arc::new(FunctionTool::new(name.clone(), move |json| {
        let input: I = serde_json::from_str(json)?;
        let output = f(input);
        Ok(serde_json::to_string(&output)?)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_binding() {
        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestParams {
            message: String,
            count: u32,
        }

        let params = TestParams {
            message: "hello".to_string(),
            count: 42,
        };

        let json = JsonBinding::params_to_json(&params).unwrap();
        let decoded: TestParams = JsonBinding::json_to_params(&json).unwrap();

        assert_eq!(params, decoded);
    }

    #[test]
    fn test_tool_registry() {
        let tool = typed_tool("echo", |input: String| -> String {
            format!("echo: {}", input)
        });

        let mut registry = ToolRegistry::new();
        registry.register("echo".to_string(), tool);

        assert!(registry.has_tool("echo"));
        assert_eq!(registry.list_tools(), vec!["echo"]);

        let result = registry.invoke("echo", r#""hello""#).unwrap();
        assert_eq!(result, r#""echo: hello""#);
    }

    #[test]
    fn test_function_tool() {
        let tool = FunctionTool::new("double".to_string(), |json: &str| {
            let n: i32 = serde_json::from_str(json)?;
            Ok(serde_json::to_string(&(n * 2))?)
        });

        let result = tool.invoke_json("21").unwrap();
        assert_eq!(result, "42");
    }
}
