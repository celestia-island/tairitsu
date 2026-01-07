//! Dynamic JSON Invocation Example
//!
//! This example demonstrates how to use JSON serialization layer for fully dynamic
//! WIT function calls, suitable for RPC, plugin systems, and other scenarios.

use anyhow::Result;
use log::{debug, error, info, warn};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use tairitsu::{JsonBinding, ToolRegistry, json::Tool, typed_tool};

// ============================================================================
// Define Tool Data Types
// ============================================================================

#[derive(Debug, Deserialize, Serialize)]
struct FsReadInput {
    path: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct FsWriteInput {
    path: String,
    data: String,
}

#[derive(Debug, Serialize)]
struct FsReadOutput {
    content: String,
    size: usize,
}

#[derive(Debug, Serialize)]
struct FsWriteOutput {
    bytes_written: usize,
    success: bool,
}

#[derive(Debug, Deserialize)]
struct CalculatorInput {
    a: f64,
    b: f64,
    operation: String,
}

#[derive(Debug, Serialize)]
struct CalculatorOutput {
    result: f64,
}

#[derive(Debug, Deserialize)]
struct StringInput {
    text: String,
    operation: String,
}

#[derive(Debug, Serialize)]
struct StringOutput {
    result: String,
}

// ============================================================================
// Implement Custom Tools
// ============================================================================

struct StringTools;

impl StringTools {
    fn process(input: StringInput) -> StringOutput {
        let result = match input.operation.as_str() {
            "upper" => input.text.to_uppercase(),
            "lower" => input.text.to_lowercase(),
            "reverse" => input.text.chars().rev().collect(),
            "length" => input.text.len().to_string(),
            _ => format!("Unknown operation: {}", input.operation),
        };

        StringOutput { result }
    }
}

// ============================================================================
// Custom Tool Implementation
// ============================================================================

struct CalculatorTool;

impl Tool for CalculatorTool {
    fn invoke_json(&self, json: &str) -> Result<String> {
        let input: CalculatorInput = serde_json::from_str(json)?;

        let result = match input.operation.as_str() {
            "add" => input.a + input.b,
            "sub" => input.a - input.b,
            "mul" => input.a * input.b,
            "div" => {
                if input.b != 0.0 {
                    input.a / input.b
                } else {
                    return Err(anyhow::anyhow!("Division by zero"));
                }
            }
            _ => return Err(anyhow::anyhow!("Unknown operation: {}", input.operation)),
        };

        let output = CalculatorOutput { result };
        Ok(serde_json::to_string(&output)?)
    }

    fn name(&self) -> &str {
        "calculator"
    }
}

// ============================================================================
// Main Function
// ============================================================================

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting dynamic JSON invocation example");

    // Scenario 1: Basic JSON Serialization
    debug!("Testing JSON serialization");
    let input = FsWriteInput {
        path: "/data/config.json".to_string(),
        data: "{\"name\":\"tairitsu\"}".to_string(),
    };

    let json = JsonBinding::params_to_json(&input)?;
    debug!("Serialized JSON: {}", json);

    let decoded: FsWriteInput = JsonBinding::json_to_params(&json)?;
    debug!("Deserialized: path={}, data={}", decoded.path, decoded.data);

    // Scenario 2: Create Type-Safe Tools
    info!("Creating type-safe tools");
    let fs_read_tool = typed_tool("fs-read", |input: FsReadInput| -> FsReadOutput {
        FsReadOutput {
            content: format!("Content of {}", input.path),
            size: 100,
        }
    });

    let fs_write_tool = typed_tool("fs-write", |input: FsWriteInput| -> FsWriteOutput {
        FsWriteOutput {
            bytes_written: input.data.len(),
            success: true,
        }
    });

    let string_tool = typed_tool("string-process", |input: StringInput| -> StringOutput {
        StringTools::process(input)
    });

    debug!("Created tools: fs-read, fs-write, string-process");

    // Scenario 3: Register and Manage Tools
    info!("Registering tools in registry");
    let mut registry = ToolRegistry::new();
    registry.register("fs-read".to_string(), fs_read_tool);
    registry.register("fs-write".to_string(), fs_write_tool);
    registry.register("string-process".to_string(), string_tool);
    registry.register("calculator".to_string(), Arc::new(CalculatorTool));

    let tools = registry.list_tools();
    info!("Registered {} tools: {}", tools.len(), tools.join(", "));

    // Scenario 4: Dynamic Tool Invocation
    info!("Testing dynamic tool invocation");

    // Test file write
    debug!("Testing fs-write");
    match registry.invoke("fs-write", r#"{"path":"/test.txt","data":"Hello, World!"}"#) {
        Ok(output) => info!("fs-write result: {}", output),
        Err(e) => error!("fs-write failed: {}", e),
    }

    // Test file read
    debug!("Testing fs-read");
    match registry.invoke("fs-read", r#"{"path":"/test.txt"}"#) {
        Ok(output) => info!("fs-read result: {}", output),
        Err(e) => error!("fs-read failed: {}", e),
    }

    // Test string processing
    debug!("Testing string-process");
    match registry.invoke("string-process", r#"{"text":"Hello","operation":"upper"}"#) {
        Ok(output) => info!("string-process result: {}", output),
        Err(e) => error!("string-process failed: {}", e),
    }

    // Test calculator
    debug!("Testing calculator");
    match registry.invoke("calculator", r#"{"a":10,"b":5,"operation":"mul"}"#) {
        Ok(output) => info!("calculator result: {}", output),
        Err(e) => error!("calculator failed: {}", e),
    }

    // Scenario 5: Error Handling
    info!("Testing error handling");

    // Test non-existent tool
    match registry.invoke("non-existent", "{}") {
        Ok(_) => warn!("Non-existent tool unexpectedly succeeded"),
        Err(e) => debug!("Expected error for non-existent tool: {}", e),
    }

    // Test invalid JSON
    match registry.invoke("fs-read", "invalid json") {
        Ok(_) => warn!("Invalid JSON unexpectedly succeeded"),
        Err(e) => debug!("Expected error for invalid JSON: {}", e),
    }

    // Test division by zero
    match registry.invoke("calculator", r#"{"a":10,"b":0,"operation":"div"}"#) {
        Ok(_) => warn!("Division by zero unexpectedly succeeded"),
        Err(e) => debug!("Expected error for division by zero: {}", e),
    }

    // Scenario 6: Random Data Testing
    info!("=== Random Data Testing ===");
    test_random_calculator(&registry)?;

    // Scenario 7: Function Existence Detection
    info!("=== Tool Existence Detection ===");
    test_tool_detection(&registry)?;

    info!("Example completed successfully");
    Ok(())
}

/// Test calculator with random data
fn test_random_calculator(registry: &ToolRegistry) -> Result<()> {
    let mut rng = rand::thread_rng();

    info!("Testing calculator with random data...");

    for i in 1..=5 {
        let a = rng.gen_range(1..1000);
        let b = rng.gen_range(1..1000);

        // Test addition
        let json_add = format!(r#"{{"a":{},"b":{},"operation":"add"}}"#, a, b);
        match registry.invoke("calculator", &json_add) {
            Ok(output) => {
                info!("  [{}] Random add: {} + {} = {}", i, a, b, output);
            }
            Err(e) => error!("  [{}] Random add failed: {}", i, e),
        }

        // Test multiplication
        let json_mul = format!(r#"{{"a":{},"b":{},"operation":"mul"}}"#, a, b);
        match registry.invoke("calculator", &json_mul) {
            Ok(output) => {
                info!("  [{}] Random mul: {} × {} = {}", i, a, b, output);
            }
            Err(e) => error!("  [{}] Random mul failed: {}", i, e),
        }
    }

    info!("✓ Random calculator tests passed");
    Ok(())
}

/// Test tool existence detection
fn test_tool_detection(registry: &ToolRegistry) -> Result<()> {
    let expected_tools = vec!["fs-read", "fs-write", "string-process", "calculator"];

    info!("Checking tool existence:");
    for expected in &expected_tools {
        let tools = registry.list_tools();
        let found = tools.iter().any(|t| t == expected);
        if found {
            info!("  ✓ Tool detected: {}", expected);
        } else {
            warn!("  ✗ Tool missing: {}", expected);
        }
    }

    let tools = registry.list_tools();
    info!("Total registered tools: {}", tools.len());

    Ok(())
}
