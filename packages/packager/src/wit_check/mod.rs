//! WIT-to-Glue Completeness Check
//!
//! Validates that every function in every imported interface of `browser-full.wit`
//! has a corresponding implementation in the browser-glue layer.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

#[derive(Debug)]
pub struct InterfaceCoverage {
    pub short_name: String,
    pub total_functions: usize,
    pub covered: Vec<String>,
    pub missing: Vec<String>,
}

impl InterfaceCoverage {
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }
    pub fn coverage_pct(&self) -> f32 {
        if self.total_functions == 0 {
            100.0
        } else {
            (self.covered.len() as f32 / self.total_functions as f32) * 100.0
        }
    }
}

#[derive(Debug)]
pub struct CompletenessReport {
    pub total_interfaces: usize,
    pub total_functions: usize,
    pub total_covered: usize,
    pub total_missing: usize,
    pub interfaces: Vec<InterfaceCoverage>,
    pub incomplete_interfaces: Vec<String>,
    pub glue_files_scanned: usize,
}

impl CompletenessReport {
    pub fn is_fully_covered(&self) -> bool {
        self.total_missing == 0
    }

    pub fn summary(&self) -> String {
        let pct = if self.total_functions > 0 {
            (self.total_covered as f32 / self.total_functions as f32) * 100.0
        } else {
            100.0
        };
        format!(
            "{}/{} functions covered ({:.1}%), {}/{} interfaces complete",
            self.total_covered,
            self.total_functions,
            pct,
            self.interfaces.iter().filter(|i| i.is_complete()).count(),
            self.total_interfaces,
        )
    }
}

/// Parse `browser-full.wit` to extract imported interface names.
///
/// Returns a Vec of (interface_name, source_file_path) tuples.
/// The actual function definitions must be parsed from each interface's own .wit file.
fn parse_browser_full_imports(wit_path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(wit_path)
        .with_context(|| format!("Failed to read {}", wit_path.display()))?;

    let mut imports = Vec::new();
    let mut in_world = false;
    let import_re = regex::Regex::new(r"^\s*import\s+([\w-]+)\s*;").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("world ") {
            in_world = true;
            continue;
        }
        if trimmed == "}" && in_world {
            break;
        }
        if !in_world {
            continue;
        }

        // Match: `import event-callbacks;`
        if let Some(cap) = import_re.captures(trimmed) {
            if let Some(name) = cap.get(1) {
                imports.push(name.as_str().to_string());
            }
        }
    }

    if imports.is_empty() {
        bail!("No imports found in {}", wit_path.display());
    }

    Ok(imports)
}

/// Parse a single interface .wit file to extract its function names.
fn parse_interface_functions(wit_path: &Path) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(wit_path)
        .with_context(|| format!("Failed to read {}", wit_path.display()))?;

    let mut funcs = Vec::new();
    let mut in_interface = false;
    let func_re = regex::Regex::new(r"^([\w-]+)\s*:\s*func\s*\(").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("interface ") {
            in_interface = true;
            continue;
        }
        if trimmed == "}" && in_interface {
            in_interface = false;
            continue;
        }
        if !in_interface {
            continue;
        }

        // Match WIT function syntax: `func-name: func(params) -> result;`
        if let Some(cap) = func_re.captures(trimmed) {
            if let Some(name) = cap.get(1) {
                let n = name.as_str();
                if !matches!(n, "constructor" | "<clinit>" | "<drop>") {
                    funcs.push(n.to_string());
                }
            }
        }
    }

    Ok(funcs)
}

/// Pre-scan all WIT files and build an index: interface_name → file_path
fn build_wit_index(wit_base: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();

    // Scan composed/ directory
    let iface_re = regex::Regex::new(r"interface\s+([\w-]+)\s*\{").unwrap();
    if let Ok(entries) = std::fs::read_dir(wit_base.join("composed")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "wit").unwrap_or(false) {
                if let Ok(c) = std::fs::read_to_string(&p) {
                    for cap in iface_re.captures_iter(&c) {
                        if let Some(name) = cap.get(1) {
                            index.insert(name.as_str().to_string(), p.clone());
                        }
                    }
                }
            }
        }
    }

    // Scan handwritten/ directory
    if let Ok(entries) = std::fs::read_dir(wit_base.join("handwritten")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "wit").unwrap_or(false) {
                if let Ok(c) = std::fs::read_to_string(&p) {
                    for cap in iface_re.captures_iter(&c) {
                        if let Some(name) = cap.get(1) {
                            index.insert(name.as_str().to_string(), p.clone());
                        }
                    }
                }
            }
        }
    }

    // Scan generated/ directory
    if let Ok(entries) = std::fs::read_dir(wit_base.join("generated")) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "wit").unwrap_or(false) {
                if let Some(stem) = p.file_stem() {
                    let name = stem.to_string_lossy().replace('_', "-");
                    index.insert(name, p);
                }
            }
        }
    }

    index
}

/// Find the .wit file that defines a given interface name.
/// Uses pre-built index for O(1) lookup.
fn find_interface_file<'a>(
    interface_name: &str,
    wit_index: &'a HashMap<String, PathBuf>,
) -> Option<&'a PathBuf> {
    wit_index.get(interface_name)
}

pub fn check_completeness(
    workspace_root: &Path,
    glue_runtime_path: Option<&Path>,
    glue_src_dir: Option<&Path>,
) -> Result<CompletenessReport> {
    let wit_file = workspace_root.join("packages/browser-worlds/wit/composed/browser-full.wit");
    let wit_base = workspace_root.join("packages/browser-worlds/wit");

    if !wit_file.exists() {
        bail!("WIT file not found: {}", wit_file.display());
    }

    // Step 1: Get list of imported interface names from browser-full.wit
    let imported_interfaces = parse_browser_full_imports(&wit_file)?;

    // Step 2: Build index of all interface definitions
    eprintln!("[ INFO ] Building WIT file index...");
    let wit_index = build_wit_index(&wit_base);
    eprintln!(
        "[ INFO ] Indexed {} interfaces from WIT files",
        wit_index.len()
    );

    // Step 3: For each imported interface, extract functions
    let mut wit_funcs: Vec<(String, Vec<String>)> = Vec::new();
    for (i, iface_name) in imported_interfaces.iter().enumerate() {
        if i % 100 == 0 {
            eprintln!(
                "[ INFO ] Processing {}/{}: {}",
                i + 1,
                imported_interfaces.len(),
                iface_name
            );
        }
        match find_interface_file(iface_name, &wit_index) {
            Some(file_path) => {
                let funcs = parse_interface_functions(file_path)?;
                wit_funcs.push((iface_name.clone(), funcs));
            }
            None => {
                eprintln!(
                    "[WARN] Could not find WIT file for interface '{}'",
                    iface_name
                );
                wit_funcs.push((iface_name.clone(), Vec::new()));
            }
        }
    }

    // Scan glue implementations
    let default_glue_src = workspace_root.join("packages/browser-glue/src/glue");
    let default_glue_runtime = workspace_root.join("packages/browser-glue/dist/runtime.js");

    let glue_dir = glue_src_dir.unwrap_or(default_glue_src.as_path());
    let runtime_path = glue_runtime_path.unwrap_or(default_glue_runtime.as_path());

    let (mut glue_exports, files_scanned) = scan_glue_exports(glue_dir, runtime_path)?;

    // Also add runtime registry exports
    let registry_path = workspace_root.join("packages/browser-glue/src/runtime/registry.ts");
    if registry_path.exists() {
        glue_exports.extend(extract_ts_exports(&registry_path)?);
    }

    // Cross-reference
    let mut interfaces = Vec::new();
    let mut total_covered = 0;
    let mut total_missing = 0;
    let mut incomplete = Vec::new();

    let mut sorted: Vec<_> = wit_funcs.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (iface_name, wit_funcs) in sorted {
        let short_name = iface_name
            .rsplit('@')
            .next()
            .unwrap_or(iface_name.as_str())
            .rsplit('/')
            .next_back()
            .unwrap_or(iface_name.as_str())
            .to_string();

        let mut covered = Vec::new();
        let mut missing = Vec::new();

        for func in &wit_funcs {
            if glue_exports.contains(func.as_str()) {
                covered.push(func.clone());
                total_covered += 1;
            } else {
                missing.push(func.clone());
                total_missing += 1;
            }
        }

        if !missing.is_empty() {
            incomplete.push(short_name.clone());
        }

        interfaces.push(InterfaceCoverage {
            short_name,
            total_functions: wit_funcs.len(),
            covered,
            missing,
        });
    }

    Ok(CompletenessReport {
        total_interfaces: interfaces.len(),
        total_functions: total_covered + total_missing,
        total_covered,
        total_missing,
        interfaces,
        incomplete_interfaces: incomplete,
        glue_files_scanned: files_scanned,
    })
}

fn scan_glue_exports(glue_dir: &Path, runtime_bundle: &Path) -> Result<(HashSet<String>, usize)> {
    let mut all_exports = HashSet::new();
    let mut file_count = 0;

    if glue_dir.is_dir() {
        for entry in walkdir::WalkDir::new(glue_dir).sort_by_file_name() {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().map(|e| e != "ts").unwrap_or(true) {
                continue;
            }
            all_exports.extend(extract_ts_exports_from_str(
                &std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read {}", path.display()))?,
            ));
            file_count += 1;
        }
    }

    if runtime_bundle.exists() {
        all_exports.extend(extract_js_runtime_exports(
            &std::fs::read_to_string(runtime_bundle)
                .with_context(|| format!("Failed to read {}", runtime_bundle.display()))?,
        ));
        file_count += 1;
    }

    Ok((all_exports, file_count))
}

fn extract_ts_exports(path: &Path) -> Result<HashSet<String>> {
    Ok(extract_ts_exports_from_str(&std::fs::read_to_string(path)?))
}

fn extract_ts_exports_from_str(content: &str) -> HashSet<String> {
    let mut exports = HashSet::new();

    let re = regex::Regex::new(r#"export\s+(?:function|const|var|let)\s+(\w+)"#).unwrap();
    for cap in re.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            exports.insert(name.as_str().to_string());
        }
    }

    let method_re = regex::Regex::new(r"(?m)^\s+(\w+)\s*\([^)]*\)\s*\{").unwrap();
    for cap in method_re.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            let n = name.as_str();
            if is_method_name(n) {
                exports.insert(n.to_string());
            }
        }
    }

    exports
}

fn is_method_name(s: &str) -> bool {
    let first = match s.chars().next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_lowercase() || s.len() < 2 {
        return false;
    }
    !matches!(
        s,
        "if" | "for"
            | "while"
            | "switch"
            | "catch"
            | "function"
            | "class"
            | "return"
            | "typeof"
            | "import"
            | "export"
            | "from"
            | "const"
            | "let"
            | "var"
            | "new"
            | "throw"
            | "try"
            | "else"
            | "in"
            | "of"
            | "do"
            | "case"
            | "default"
            | "break"
            | "continue"
            | "yield"
            | "async"
            | "await"
            | "this"
            | "super"
            | "null"
            | "undefined"
            | "true"
            | "false"
    )
}

fn extract_js_runtime_exports(content: &str) -> HashSet<String> {
    let mut exports = HashSet::new();

    // Match: location_exports = { ... } or similar patterns
    // Use a simpler pattern that finds the object body
    let obj_re = regex::Regex::new(r"(\w+)_exports\s*=\s*\{").unwrap();
    for cap in obj_re.captures_iter(content) {
        if let Some(prefix) = cap.get(1) {
            // Now find all method assignments in this object
            let method_re =
                regex::Regex::new(&format!(r"{}\s*:\s*function\s+(\w+)", prefix.as_str())).unwrap();
            for m in method_re.captures_iter(content) {
                if let Some(name) = m.get(1) {
                    exports.insert(name.as_str().to_string());
                }
            }
            // Also match shorthand methods: name(params)
            let short_re =
                regex::Regex::new(&format!(r"{}\s*\{{\s*(\w+)\s*\(", prefix.as_str())).unwrap();
            for m in short_re.captures_iter(content) {
                if let Some(name) = m.get(1) {
                    exports.insert(name.as_str().to_string());
                }
            }
        }
    }

    let fn_re = regex::Regex::new(r"^\s+function\s+(\w+)\s*\(").unwrap();
    for cap in fn_re.captures_iter(content) {
        if let Some(name) = cap.get(1) {
            exports.insert(name.as_str().to_string());
        }
    }

    exports
}

pub fn print_report(report: &CompletenessReport, verbose: bool) {
    println!("\n{}", "=".repeat(60));
    println!("  WIT-to-Glue Completeness Report");
    println!("{}", "=".repeat(60));
    println!("\n  {}", report.summary());
    println!("  Glue source files scanned: {}", report.glue_files_scanned);

    if report.is_fully_covered() {
        println!("\n  ✅ All WIT interface functions have glue implementations.\n");
        return;
    }

    println!(
        "\n  ❌ {} MISSING function(s) across {} interface(s):\n",
        report.total_missing,
        report.incomplete_interfaces.len()
    );

    for iface in &report.interfaces {
        if iface.is_complete() && !verbose {
            continue;
        }
        let status = if iface.is_complete() { "✅" } else { "❌" };
        println!(
            "  {} {} ({}/{} — {:.0}%)",
            status,
            iface.short_name,
            iface.covered.len(),
            iface.total_functions,
            iface.coverage_pct(),
        );
        for m in &iface.missing {
            println!("      ✗  {}()", m);
        }
        if verbose && !iface.covered.is_empty() {
            for c in &iface.covered {
                println!("      ✓  {}()", c);
            }
        }
    }

    println!("\n  Fix: Add missing methods to packages/browser-glue/src/runtime/*.ts");
    println!("       Then run: cd packages/browser-glue && npm run build\n");
}
