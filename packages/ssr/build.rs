//! SSR stub code generator
//!
//! This build script parses the WIT file and generates stub implementations
//! for all browser interfaces that are not manually implemented.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    // Only regenerate if the WIT file changes
    println!("cargo:rerun-if-changed=../../browser-worlds/wit/browser-full.wit");

    // Read the WIT file to extract interface names and functions
    let wit_path = "../../browser-worlds/wit/browser-full.wit";
    let wit_content = match std::fs::read_to_string(wit_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read WIT file: {}", e);
            eprintln!("SSR stubs will not be generated");
            return;
        }
    };

    // Interfaces that are manually implemented (don't generate stubs for these)
    let manual_interfaces: HashSet<&'static str> = [
        "platform-helpers",
        "console",
        "event-target",
        "style",
        "document",
        "node",
        "element",
        "window",
        "types",
    ]
    .into_iter()
    .collect();

    // Parse the WIT file to find all interfaces and their functions
    let interfaces = parse_wit_interfaces(&wit_content, &manual_interfaces);

    // Generate the stub code
    let generated_code = generate_stubs(&interfaces);

    // Write to the generated file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let generated_path = Path::new(&out_dir).join("ssr_stubs.rs");
    let mut file = File::create(&generated_path).expect("Failed to create generated file");
    file.write_all(generated_code.as_bytes())
        .expect("Failed to write generated code");

    println!("Generated stub implementations for {} interfaces", interfaces.len());
}

/// Represents a function signature in a WIT interface
struct WitFunction {
    name: String,
    params: Vec<(String, String)>,
    return_type: Option<String>,
}

/// Represents a WIT interface
struct WitInterface {
    name: String,
    functions: Vec<WitFunction>,
}

/// Parse the WIT file to extract interfaces and their functions
fn parse_wit_interfaces(wit_content: &str, manual_interfaces: &HashSet<&str>) -> Vec<WitInterface> {
    let mut interfaces = Vec::new();
    let mut current_interface: Option<WitInterface> = None;
    let mut in_interface = false;

    for line in wit_content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with("///") || line.starts_with("//") {
            continue;
        }

        // Check for interface definition
        if line.starts_with("interface ") {
            // Extract interface name
            let rest = &line[10..]; // Skip "interface "
            let name = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.split('{').next())
                .unwrap_or("");

            if !name.is_empty() && !manual_interfaces.contains(name) {
                in_interface = true;
                current_interface = Some(WitInterface {
                    name: name.to_string(),
                    functions: Vec::new(),
                });
            } else {
                in_interface = false;
                current_interface = None;
            }
        } else if line == "}" && in_interface {
            // End of interface
            if let Some(interface) = current_interface.take() {
                interfaces.push(interface);
            }
            in_interface = false;
        } else if in_interface && line.contains(": func") {
            // Parse function definition
            if let Some(interface) = &mut current_interface {
                if let Some(func) = parse_function(line) {
                    interface.functions.push(func);
                }
            }
        }
    }

    // Don't forget the last interface
    if let Some(interface) = current_interface {
        interfaces.push(interface);
    }

    interfaces
}

/// Parse a function definition from a WIT line
fn parse_function(line: &str) -> Option<WitFunction> {
    // Format: "function-name: func(param1: type1, param2: type2) -> return-type;"
    // or: "function-name: func(param1: type1, param2: type2);"

    let func_part = line.split(": func").collect::<Vec<_>>();
    if func_part.len() != 2 {
        return None;
    }

    let name = func_part[0].trim().to_string();
    let signature = func_part[1].trim().trim_end_matches(';');

    // Parse return type
    let (params_part, return_type) = if let Some(idx) = signature.find("->") {
        let ret = signature[idx + 2..].trim().to_string();
        (&signature[..idx], Some(ret))
    } else {
        (signature, None)
    };

    // Parse parameters
    let params = if params_part.trim() == "()" {
        Vec::new()
    } else {
        let inner = params_part.trim().trim_start_matches('(').trim_end_matches(')');
        inner
            .split(',')
            .filter_map(|p| {
                let parts: Vec<&str> = p.trim().split(':').collect();
                if parts.len() == 2 {
                    Some((parts[0].trim().to_string(), parts[1].trim().to_string()))
                } else {
                    None
                }
            })
            .collect()
    };

    Some(WitFunction {
        name,
        params,
        return_type,
    })
}

/// Generate the stub implementations code
fn generate_stubs(interfaces: &[WitInterface]) -> String {
    let mut code = String::new();

    code.push_str("// Auto-generated SSR stub implementations\n");
    code.push_str("// Generated by build.rs from browser-full.wit\n");
    code.push_str("// DO NOT EDIT\n\n");

    code.push_str("use crate::host_state::SsrHostState;\n");
    code.push_str("use anyhow::Result;\n");
    code.push_str("use wasmtime::component::Linker;\n\n");

    // Generate individual interface registration functions
    for interface in interfaces {
        code.push_str(&generate_interface_registration(interface));
    }

    // Generate the top-level registration function
    code.push_str("\n/// Register all stub implementations with the linker\n");
    code.push_str("pub fn register_all_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {\n");

    for interface in interfaces {
        let fn_name = sanitize_identifier(&interface.name);
        code.push_str(&format!("    register_{}_stubs(linker)?;\n", fn_name));
    }

    code.push_str("    Ok(())\n");
    code.push_str("}\n");

    code
}

/// Generate registration function for a single interface
fn generate_interface_registration(interface: &WitInterface) -> String {
    let mut code = String::new();
    let fn_name = sanitize_identifier(&interface.name);
    let wit_name = &interface.name;

    code.push_str(&format!("fn register_{}_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {{\n", fn_name));
    code.push_str(&format!("    let mut instance = linker.instance(\"tairitsu-browser:full/{}@0.2.0\")?;\n", wit_name));

    if interface.functions.is_empty() {
        code.push_str("    // No functions to stub\n");
        code.push_str("    let _ = instance;\n");
    } else {
        for func in &interface.functions {
            code.push_str(&generate_function_stub(func, wit_name));
        }
        code.push_str("    let _ = instance;\n");
    }

    code.push_str("    Ok(())\n");
    code.push_str("}\n\n");

    code
}

/// Generate stub implementation for a single function
fn generate_function_stub(func: &WitFunction, interface_name: &str) -> String {
    let mut code = String::new();
    let func_name = &func.name;

    // Generate parameter names and types
    let param_names: Vec<String> = func.params.iter()
        .enumerate()
        .map(|(i, (name, _))| {
            if name == "_" || name.is_empty() {
                format!("_arg{}", i)
            } else {
                format!("_{}", name)
            }
        })
        .collect();

    let params_tuple = if param_names.is_empty() {
        "()".to_string()
    } else {
        format!("({},)", param_names.join(", "))
    };

    // Generate the return type based on the function signature
    let return_type = if let Some(ret) = &func.return_type {
        match ret.as_str() {
            "" => "()".to_string(),
            _ => {
                // Handle result<T, E> types
                if ret.starts_with("result<") {
                    // Extract the success type
                    let inner = ret.strip_prefix("result<").unwrap_or(ret)
                        .strip_suffix(">").unwrap_or(ret);
                    if inner == "_" {
                        "Result<(), String>".to_string()
                    } else {
                        // Map WIT types to Rust types
                        format!("Result<{}, String>", map_wit_type_to_rust(inner))
                    }
                } else if ret.starts_with("option<") {
                    let inner = ret.strip_prefix("option<").unwrap_or(ret)
                        .strip_suffix(">").unwrap_or(ret);
                    format!("Option<{}>", map_wit_type_to_rust(inner))
                } else {
                    map_wit_type_to_rust(ret)
                }
            }
        }
    } else {
        "()".to_string()
    };

    // Generate the return value
    let return_stmt = if let Some(ret) = &func.return_type {
        if ret.starts_with("result<") {
            format!("Ok(({},))", get_default_for_return_type(ret))
        } else if ret.starts_with("option<") {
            format!("(({},))", get_default_for_return_type(ret))
        } else if ret == "_" {
            format!("Ok(())")
        } else {
            let default = get_default_for_return_type(ret);
            if return_type == "()" {
                format!("Ok(())")
            } else {
                format!("Ok(({},))", default)
            }
        }
    } else {
        "Ok(())".to_string()
    };

    code.push_str(&format!(
        "    instance.func_wrap(\n\
          \"{}\",\n\
          |_caller, {}: ()| -> Result<{}, wasmtime::Error> {{\n\
          {}\n\
          }},\n\
          )?;\n",
        func_name,
        params_tuple,
        return_type,
        return_stmt
    ));

    code
}

/// Map WIT types to Rust types
fn map_wit_type_to_rust(wit_type: &str) -> String {
    // Handle tuple types like "(f64, f64, f64, f64)"
    if wit_type.starts_with('(') && wit_type.ends_with(')') {
        let inner = &wit_type[1..wit_type.len()-1];
        let types: Vec<String> = inner.split(',')
            .map(|t| map_wit_type_to_rust(t.trim()))
            .collect();
        return format!("({})", types.join(", "));
    }

    match wit_type {
        "bool" => "bool".to_string(),
        "s8" => "i8".to_string(),
        "s16" => "i16".to_string(),
        "s32" => "i32".to_string(),
        "s64" => "i64".to_string(),
        "u8" => "u8".to_string(),
        "u16" => "u16".to_string(),
        "u32" => "u32".to_string(),
        "u64" => "u64".to_string(),
        "f32" => "f32".to_string(),
        "f64" => "f64".to_string(),
        "string" => "String".to_string(),
        "char" => "char".to_string(),
        "_" => "()".to_string(),
        t if t.starts_with("list<") => {
            let inner = t.strip_prefix("list<").unwrap_or(t)
                .strip_suffix(">").unwrap_or(t);
            format!("Vec<{}>", map_wit_type_to_rust(inner))
        }
        t => t.to_string(), // Pass through unknown types
    }
}

/// Get default value for a return type
fn get_default_for_return_type(wit_type: &str) -> String {
    if wit_type.starts_with("result<") {
        // Return error for result types
        return "Err(\"\\\"{interface}.{function}\\\" is not available in SSR. Use use_effect() for browser-only operations.\".to_string())".to_string();
    }

    if wit_type.starts_with("option<") {
        return "None".to_string();
    }

    let inner = wit_type.strip_prefix("result<").unwrap_or(wit_type)
        .strip_suffix(">").unwrap_or(wit_type);

    match inner {
        "bool" => "false".to_string(),
        "s8" | "s16" | "s32" | "s64" => "0".to_string(),
        "u8" | "u16" | "u32" | "u64" => "0".to_string(),
        "f32" | "f64" => "0.0".to_string(),
        "string" => "String::new()".to_string(),
        "char" => "'\\0'".to_string(),
        "_" => "()".to_string(),
        t if t.starts_with("list<") => "Vec::new()".to_string(),
        t if t.starts_with('(') && t.ends_with(')') => {
            // Handle tuples
            let inner = &t[1..t.len()-1];
            let defaults: Vec<String> = inner.split(',')
                .map(|typ| get_default_for_return_type(typ.trim()))
                .collect();
            format!("({})", defaults.join(", "))
        }
        t => "0".to_string(), // Default for unknown types
    }
}

/// Sanitize an identifier for use as a Rust function name
fn sanitize_identifier(name: &str) -> String {
    name.replace('-', "_")
}
