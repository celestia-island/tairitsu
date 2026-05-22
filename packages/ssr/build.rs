//! SSR stub code generator
//!
//! This build script parses the WIT file and generates stub implementations
//! for all browser interfaces that are not manually implemented.

use std::{collections::HashSet, fs::File, io::Write, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=wit/browser-full.wit");
    println!("cargo:rerun-if-changed=wit/composed");

    let wit_path = "wit/browser-full.wit";
    let wit_content = match std::fs::read_to_string(wit_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Warning: Could not read WIT file: {}", e);
            eprintln!("SSR stubs will not be generated, using minimal stubs");

            // Generate a minimal stub file anyway
            let out_dir = std::env::var("OUT_DIR").unwrap();
            let generated_path = Path::new(&out_dir).join("ssr_stubs_gen.rs");
            let mut file = File::create(&generated_path).expect("Failed to create generated file");
            let minimal_code =
                "// Minimal SSR stubs (WIT file not found)\npub fn register_all_auto_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> { Ok(()) }\n".to_string();
            file.write_all(minimal_code.as_bytes())
                .expect("Failed to write generated code");
            println!("cargo:warning=Generated minimal stub implementations (WIT file not found)");
            return;
        }
    };

    // Interfaces that are manually implemented (don't generate stubs for these)
    let manual_interfaces: HashSet<&'static str> = [
        "platform-helpers",
        "console",
        "event-target",
        "event",
        "document",
        "node",
        "element",
        "window",
        "types",
        "component-types",
        // CSS interfaces (manually implemented in linker.rs)
        "element-css-inline-style",
        "css-style-declaration",
        // Credential management interfaces (manually implemented in stubs.rs)
        "credential",
        "credentials-container",
        "credential-user-data",
        "federated-credential",
        "password-credential",
        // History interface (manually implemented in stubs.rs)
        "history",
        // Media decoder/encoder interfaces (manually implemented in stubs.rs)
        "audio-decoder",
        "audio-encoder",
        "video-decoder",
        "video-encoder",
        // Resize observer interfaces (implemented via bindgen in bindings.rs)
        "resize-observer",
        "resize-observer-entry",
        "resize-observer-size",
        // Callback interfaces (implemented by the component, not the host)
        "timer-callbacks",
        "animation-callbacks",
        "resize-observer-callbacks",
        "mutation-observer-callbacks",
        "media-query-list-callbacks",
        "scroll-callbacks",
        "window-resize-callbacks",
        "video-frame-callbacks",
        "event-callbacks",
        "lifecycle",
        "promise-callbacks",
        "geolocation-callbacks",
        "idb-callbacks",
        "file-reader-callbacks",
    ]
    .into_iter()
    .collect();

    // Parse the WIT file to find all interfaces and their functions
    let interfaces = parse_wit_interfaces(&wit_content, &manual_interfaces);

    // Generate the stub code
    let generated_code = generate_stubs(&interfaces);

    // Write to the generated file
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let generated_path = Path::new(&out_dir).join("ssr_stubs_gen.rs");
    let mut file = File::create(&generated_path).expect("Failed to create generated file");
    file.write_all(generated_code.as_bytes())
        .expect("Failed to write generated code");

    println!(
        "cargo:warning=Generated {} stub interfaces",
        interfaces.len()
    );
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
        if let Some(rest) = line.strip_prefix("interface ") {
            // Extract interface name
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
        let inner = params_part
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')');
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

    // Don't include use statements here to avoid conflicts with stubs.rs
    // code.push_str("use crate::host_state::SsrHostState;\n");
    // code.push_str("use anyhow::Result;\n");
    // code.push_str("use wasmtime::component::Linker;\n\n");

    // Generate individual interface registration functions
    for interface in interfaces {
        code.push_str(&generate_interface_registration(interface));
    }

    // Generate the top-level registration function
    code.push_str("\n/// Register all auto-generated stub implementations with the linker\n");
    code.push_str(
        "pub fn register_all_auto_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {\n",
    );

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

    if interface.functions.is_empty() {
        // Empty interface - just return Ok
        code.push_str(&format!(
            "fn register_{}_stubs(_linker: &mut Linker<SsrHostState>) -> Result<()> {{\n",
            fn_name
        ));
        code.push_str("    Ok(())\n");
        code.push_str("}\n\n");
    } else {
        code.push_str(&format!(
            "fn register_{}_stubs(linker: &mut Linker<SsrHostState>) -> Result<()> {{\n",
            fn_name
        ));
        code.push_str(&format!(
            "    let mut instance = linker.instance(\"tairitsu-browser:full/{}@0.2.0\")?;\n",
            wit_name
        ));

        for func in &interface.functions {
            code.push_str(&generate_function_stub(func, wit_name));
        }

        code.push_str("    let _ = instance;\n");
        code.push_str("    Ok(())\n");
        code.push_str("}\n\n");
    }

    code
}

/// Generate stub implementation for a single function
fn generate_function_stub(func: &WitFunction, _interface_name: &str) -> String {
    let mut code = String::new();
    let func_name = &func.name;

    // Generate parameter names
    let param_names: Vec<String> = func
        .params
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            if name == "_" || name.is_empty() || name.starts_with('%') {
                // Handle reserved keywords and special names
                format!("_arg{}", i)
            } else {
                // Replace hyphens with underscores to make valid Rust identifiers
                let sanitized = name.replace('-', "_");
                format!("_{}", sanitized)
            }
        })
        .collect();

    let params_tuple = if param_names.is_empty() {
        "()".to_string()
    } else {
        format!("({},)", param_names.join(", "))
    };

    // Generate parameter types
    let param_types: Vec<String> = func
        .params
        .iter()
        .map(|(_, wit_type)| map_wit_type_to_rust(wit_type))
        .collect();

    let param_types_tuple = if param_types.is_empty() {
        "()".to_string()
    } else {
        format!("({},)", param_types.join(", "))
    };

    // Generate the return type based on the function signature
    let return_type = if let Some(ret) = &func.return_type {
        map_wit_return_type_to_rust(ret)
    } else {
        "()".to_string()
    };

    // Generate the return value
    let return_stmt = if let Some(ret) = &func.return_type {
        get_stub_return_value(ret)
    } else {
        "Ok(())".to_string()
    };

    code.push_str(&format!(
        "    instance.func_wrap(\n\
          \"{}\",\n\
          |_caller, {}: {}| -> Result<{}, wasmtime::Error> {{\n\
          {}\n\
          }},\n\
          )?;\n",
        func_name, params_tuple, param_types_tuple, return_type, return_stmt
    ));

    code
}

/// Map WIT return types to Rust types
/// wasmtime func_wrap expects all return values to be tuples
fn map_wit_return_type_to_rust(wit_type: &str) -> String {
    match wit_type {
        "" => "()".to_string(),
        t if t.starts_with("result<") => {
            // Parse result<T, E> by finding the comma that separates T and E
            // Need to handle nested types like result<list<string>, string>
            let inner = t
                .strip_prefix("result<")
                .unwrap_or(t)
                .strip_suffix(">")
                .unwrap_or(t);
            if inner == "_" {
                "(Result<(), String>,)".to_string()
            } else {
                // Find the comma that separates T and E (at depth 0)
                let mut depth = 0;
                let mut comma_idx = None;
                for (i, c) in inner.chars().enumerate() {
                    match c {
                        '<' => depth += 1,
                        '>' => depth -= 1,
                        ',' if depth == 0 => {
                            comma_idx = Some(i);
                            break;
                        }
                        _ => {}
                    }
                }
                if let Some(idx) = comma_idx {
                    let t_type = &inner[..idx];
                    let e_type = inner[idx + 1..].trim();
                    let t_rust = map_wit_type_to_rust(t_type.trim());
                    let e_rust = map_wit_type_to_rust(e_type);
                    // Wrap Result in a tuple for wasmtime
                    format!("(Result<{}, {}>,)", t_rust, e_rust)
                } else {
                    // No comma found, treat as result<T> (success only)
                    let t_rust = map_wit_type_to_rust(inner.trim());
                    format!("(Result<{}, String>,)", t_rust)
                }
            }
        }
        t if t.starts_with("option<") => {
            let inner = t
                .strip_prefix("option<")
                .unwrap_or(t)
                .strip_suffix(">")
                .unwrap_or(t);
            let inner_rust = map_wit_type_to_rust(inner);
            // Wrap Option in a tuple for wasmtime
            format!("(Option<{}>,)", inner_rust)
        }
        t => {
            let rust_type = map_wit_type_to_rust(t);
            // Wrap all types in a tuple for wasmtime
            if rust_type == "()" {
                "()".to_string()
            } else if rust_type.starts_with('(') && rust_type.ends_with(')') {
                rust_type
            } else {
                format!("({},)", rust_type)
            }
        }
    }
}

/// Map WIT types to Rust types
fn map_wit_type_to_rust(wit_type: &str) -> String {
    // Handle tuple types like "(f64, f64, f64, f64)"
    if wit_type.starts_with('(') && wit_type.ends_with(')') {
        let inner = &wit_type[1..wit_type.len() - 1];
        let types: Vec<String> = inner
            .split(',')
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
        // Handle custom handle types (e.g., mutation-record-handle -> u64)
        t if t.ends_with("-handle") => "u64".to_string(),
        // Handle common record types
        "dom-rect" => "(f64, f64, f64, f64)".to_string(),
        "mouse-event-data" => "u64".to_string(), // Simplified for SSR
        "keyboard-event-data" => "u64".to_string(),
        "focus-event-data" => "u64".to_string(),
        "input-event-data" => "u64".to_string(),
        "blob-property-bag" => "(String,)".to_string(),
        "file-property-bag" => "(i64,)".to_string(),
        t if t.starts_with("list<") => {
            let inner = t
                .strip_prefix("list<")
                .unwrap_or(t)
                .strip_suffix(">")
                .unwrap_or(t);
            format!("Vec<{}>", map_wit_type_to_rust(inner))
        }
        t if t.starts_with("option<") => {
            let inner = t
                .strip_prefix("option<")
                .unwrap_or(t)
                .strip_suffix(">")
                .unwrap_or(t);
            format!("Option<{}>", map_wit_type_to_rust(inner))
        }
        // Pass through unknown types (e.g., custom types)
        // Replace hyphens with underscores to make valid Rust identifiers
        t => t.replace('-', "_"),
    }
}

/// Get stub return value for a return type
fn get_stub_return_value(wit_type: &str) -> String {
    // wasmtime func_wrap expects Result<T, wasmtime::Error>
    // For WIT result<T, E> types, we always return an error
    if wit_type.starts_with("result<") {
        return "Err(anyhow::anyhow!(\"Browser-only operation not available in SSR\"))".to_string();
    }

    if wit_type.starts_with("option<") {
        // For option types, return None wrapped in a tuple
        return "Ok((None,))".to_string();
    }

    match wit_type {
        "bool" => "Ok((false,))".to_string(),
        "s8" | "s16" | "s32" | "s64" => "Ok((0,))".to_string(),
        "u8" | "u16" | "u32" | "u64" => "Ok((0,))".to_string(),
        "f32" | "f64" => "Ok((0.0,))".to_string(),
        "string" => "Ok((String::new(),))".to_string(),
        "char" => "Ok(('\\0',))".to_string(),
        "_" => "Ok(())".to_string(),
        // Handle common record types
        "dom-rect" => "Ok((0.0, 0.0, 0.0, 0.0))".to_string(),
        t if t.starts_with("list<") => "Ok((Vec::new(),))".to_string(),
        t if t.starts_with('(') && t.ends_with(')') => {
            // Handle tuples (e.g., (f64, f64, f64, f64))
            let inner = &t[1..t.len() - 1];
            let defaults: Vec<String> = inner
                .split(',')
                .map(|typ| get_default_value(typ.trim()))
                .collect();
            format!("Ok((({},)))", defaults.join(", "))
        }
        _t => "Ok((0,))".to_string(), // Default for unknown types (e.g., u64 handles)
    }
}

/// Get default value for a type
fn get_default_value(wit_type: &str) -> String {
    match wit_type {
        "bool" => "false".to_string(),
        "s8" | "s16" | "s32" | "s64" => "0".to_string(),
        "u8" | "u16" | "u32" | "u64" => "0".to_string(),
        "f32" | "f64" => "0.0".to_string(),
        "string" => "String::new()".to_string(),
        "char" => "'\\0'".to_string(),
        "_" => "()".to_string(),
        t if t.starts_with("list<") => "Vec::new()".to_string(),
        _t => "0".to_string(),
    }
}

/// Sanitize an identifier for use as a Rust function name
fn sanitize_identifier(name: &str) -> String {
    name.replace('-', "_")
}
