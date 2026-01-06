//! WIT definition loader and inspector
//!
//! This module provides utilities for parsing and inspecting WIT definitions
//! without generating code, useful for introspection and tooling.

use anyhow::Result;
use wit_parser::Resolve;

/// WIT definition loader
pub struct WitLoader {
    pub resolve: Resolve,
}

impl WitLoader {
    /// Load WIT definitions from a directory
    ///
    /// # Arguments
    /// * `path` - Path to directory containing WIT files
    ///
    /// # Example
    /// ```no_run
    /// use tairitsu::WitLoader;
    ///
    /// let loader = WitLoader::from_dir("./wit")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn from_dir(path: &str) -> Result<Self> {
        let mut resolve = Resolve::default();

        // Try to parse directly first
        match resolve.push_path(path) {
            Ok(_) => Ok(Self { resolve }),
            Err(e) => {
                // If that fails, try with absolute path
                match std::fs::canonicalize(path) {
                    Ok(absolute_path) => {
                        resolve.push_path(&absolute_path)
                            .map_err(|e2| {
                                anyhow::anyhow!(
                                    "Failed to parse WIT from '{}': {}\nAbsolute path '{}': {}",
                                    path, e, absolute_path.display(), e2
                                )
                            })?;
                        Ok(Self { resolve })
                    }
                    Err(_) => {
                        Err(anyhow::anyhow!(
                            "Failed to parse WIT from '{}': {}\nNote: Path may not exist or be inaccessible",
                            path, e
                        ))
                    }
                }
            }
        }
    }

    /// Load WIT definitions from a single file
    ///
    /// # Arguments
    /// * `path` - Path to WIT file
    pub fn from_file(path: &str) -> Result<Self> {
        let mut resolve = Resolve::default();

        // Try direct path first, then absolute path
        match resolve.push_path(path) {
            Ok(_) => Ok(Self { resolve }),
            Err(e) => {
                match std::fs::canonicalize(path) {
                    Ok(absolute_path) => {
                        resolve.push_path(&absolute_path)
                            .map_err(|e2| {
                                anyhow::anyhow!(
                                    "Failed to parse WIT file '{}': {}\nWith absolute path '{}': {}",
                                    path, e, absolute_path.display(), e2
                                )
                            })?;
                        Ok(Self { resolve })
                    }
                    Err(_) => {
                        Err(anyhow::anyhow!(
                            "Failed to parse WIT file '{}': {}\nNote: File may not exist or be inaccessible",
                            path, e
                        ))
                    }
                }
            }
        }
    }

    /// Get list of all world names
    ///
    /// # Returns
    /// Vector of world names in format "package:world-name"
    pub fn list_worlds(&self) -> Vec<String> {
        self.resolve
            .worlds
            .iter()
            .map(|(id, _)| {
                let world = &self.resolve.worlds[id];
                let name = &world.name;

                if let Some(pkg_id) = world.package {
                    let pkg = &self.resolve.packages[pkg_id];
                    format!("{pkg_name}/{name}", pkg_name = pkg.name)
                } else {
                    name.to_string()
                }
            })
            .collect()
    }

    /// Get export functions from a world
    ///
    /// # Arguments
    /// * `world_name` - World name in format "package:world-name"
    ///
    /// # Returns
    /// Vector of function information
    ///
    /// # Example
    /// ```no_run
    /// use tairitsu::WitLoader;
    ///
    /// let loader = WitLoader::from_dir("./wit")?;
    /// let exports = loader.list_exports("my-package:my-world");
    /// for func in exports {
    ///     println!("{}: {:?}", func.name, func.params);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn list_exports(&self, world_name: &str) -> Vec<FunctionInfo> {
        let (package_name, world_part) = world_name.split_once(':').unwrap_or(("", world_name));

        // world_part might be "namespace/world" or just "world"
        // Extract the actual world name (last part after /)
        let world_name = world_part.rsplit('/').next().unwrap_or(world_part);

        let mut functions = Vec::new();

        for (id, _) in &self.resolve.worlds {
            let world = &self.resolve.worlds[id];

            // Match package name
            if !package_name.is_empty() {
                if let Some(pkg_id) = world.package {
                    let pkg = &self.resolve.packages[pkg_id];
                    let pkg_name = format!("{}", pkg.name);

                    // pkg.name is "namespace:name" like "tairitsu:core"
                    // package_name input might be just "tairitsu" or full "tairitsu:core"
                    // So we check if pkg_name starts with package_name
                    if !pkg_name.starts_with(&format!("{}:", package_name))
                        && pkg_name != package_name
                    {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // Match world name
            if world.name == world_name {
                // Collect exports
                for (export_name, item) in &world.exports {
                    match item {
                        wit_parser::WorldItem::Function(func) => {
                            let params: Vec<(String, String)> = func
                                .params
                                .iter()
                                .map(|(name, ty)| (name.clone(), self.format_type(ty)))
                                .collect();

                            let name_str = match export_name {
                                wit_parser::WorldKey::Name(name) => name.clone(),
                                wit_parser::WorldKey::Interface(id) => {
                                    format!("interface-{}", id.index())
                                }
                            };

                            functions.push(FunctionInfo {
                                name: name_str,
                                params,
                            });
                        }
                        wit_parser::WorldItem::Interface { id, stability: _ } => {
                            // Handle exported interfaces - extract all functions from the interface
                            let interface_id = match export_name {
                                wit_parser::WorldKey::Name(_interface_name) => {
                                    // Lookup interface by name - use id from WorldItem
                                    *id
                                }
                                wit_parser::WorldKey::Interface(k) => *k,
                            };

                            if let Some(interface) = self.resolve.interfaces.get(interface_id) {
                                // Collect all functions from the interface
                                for (func_name, func_item) in &interface.functions {
                                    let params: Vec<(String, String)> = func_item
                                        .params
                                        .iter()
                                        .map(|(name, ty)| (name.clone(), self.format_type(ty)))
                                        .collect();

                                    functions.push(FunctionInfo {
                                        name: func_name.clone(),
                                        params,
                                    });
                                }
                            }
                        }
                        _ => {
                            // Ignore other types (types, etc.)
                        }
                    }
                }
                break;
            }
        }

        functions
    }

    /// Get import functions from a world
    ///
    /// # Arguments
    /// * `world_name` - World name in format "package:world-name"
    pub fn list_imports(&self, world_name: &str) -> Vec<FunctionInfo> {
        let (package_name, world_part) = world_name.split_once(':').unwrap_or(("", world_name));

        // world_part might be "namespace/world" or just "world"
        // Extract the actual world name (last part after /)
        let world_name = world_part.rsplit('/').next().unwrap_or(world_part);

        let mut functions = Vec::new();

        for (id, _) in &self.resolve.worlds {
            let world = &self.resolve.worlds[id];

            if !package_name.is_empty() {
                if let Some(pkg_id) = world.package {
                    let pkg = &self.resolve.packages[pkg_id];
                    let pkg_name = format!("{}", pkg.name);

                    // pkg.name is "namespace:name" like "tairitsu:core"
                    // package_name input might be just "tairitsu" or full "tairitsu:core"
                    // So we check if pkg_name starts with package_name
                    if !pkg_name.starts_with(&format!("{}:", package_name))
                        && pkg_name != package_name
                    {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if world.name == world_name {
                for (import_name, item) in &world.imports {
                    match item {
                        wit_parser::WorldItem::Function(func) => {
                            let params: Vec<(String, String)> = func
                                .params
                                .iter()
                                .map(|(name, ty)| (name.clone(), self.format_type(ty)))
                                .collect();

                            let name_str = match import_name {
                                wit_parser::WorldKey::Name(name) => name.clone(),
                                wit_parser::WorldKey::Interface(id) => {
                                    format!("interface-{}", id.index())
                                }
                            };

                            functions.push(FunctionInfo {
                                name: name_str,
                                params,
                            });
                        }
                        wit_parser::WorldItem::Interface { id, stability: _ } => {
                            // Handle imported interfaces - extract all functions from the interface
                            let interface_id = match import_name {
                                wit_parser::WorldKey::Name(_interface_name) => {
                                    // Lookup interface by name - use id from WorldItem
                                    *id
                                }
                                wit_parser::WorldKey::Interface(k) => *k,
                            };

                            if let Some(interface) = self.resolve.interfaces.get(interface_id) {
                                // Collect all functions from the interface
                                for (func_name, func_item) in &interface.functions {
                                    let params: Vec<(String, String)> = func_item
                                        .params
                                        .iter()
                                        .map(|(name, ty)| (name.clone(), self.format_type(ty)))
                                        .collect();

                                    functions.push(FunctionInfo {
                                        name: func_name.clone(),
                                        params,
                                    });
                                }
                            }
                        }
                        _ => {
                            // Ignore other types (types, etc.)
                        }
                    }
                }
                break;
            }
        }

        functions
    }

    /// Format a type as string
    fn format_type(&self, ty: &wit_parser::Type) -> String {
        match ty {
            wit_parser::Type::Bool => "bool".to_string(),
            wit_parser::Type::U8 => "u8".to_string(),
            wit_parser::Type::U16 => "u16".to_string(),
            wit_parser::Type::U32 => "u32".to_string(),
            wit_parser::Type::U64 => "u64".to_string(),
            wit_parser::Type::S8 => "i8".to_string(),
            wit_parser::Type::S16 => "i16".to_string(),
            wit_parser::Type::S32 => "i32".to_string(),
            wit_parser::Type::S64 => "i64".to_string(),
            wit_parser::Type::F32 => "f32".to_string(),
            wit_parser::Type::F64 => "f64".to_string(),
            wit_parser::Type::Char => "char".to_string(),
            wit_parser::Type::String => "string".to_string(),
            wit_parser::Type::ErrorContext => "error_context".to_string(),
            wit_parser::Type::Id(id) => {
                let type_def = &self.resolve.types[*id];
                match &type_def.kind {
                    wit_parser::TypeDefKind::List(ty) => {
                        format!("List<{}>", self.format_type(ty))
                    }
                    wit_parser::TypeDefKind::Option(ty) => {
                        format!("Option<{}>", self.format_type(ty))
                    }
                    wit_parser::TypeDefKind::Result(r) => {
                        let ok =
                            r.ok.as_ref()
                                .map(|ty| self.format_type(ty))
                                .unwrap_or_else(|| "()".to_string());
                        let err = r
                            .err
                            .as_ref()
                            .map(|ty| self.format_type(ty))
                            .unwrap_or_else(|| "()".to_string());
                        format!("Result<{}, {}>", ok, err)
                    }
                    wit_parser::TypeDefKind::Tuple(t) => {
                        let types: Vec<String> =
                            t.types.iter().map(|ty| self.format_type(ty)).collect();
                        format!("({})", types.join(", "))
                    }
                    wit_parser::TypeDefKind::Type(ty) => self.format_type(ty),
                    _ => type_def
                        .name
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string()),
                }
            }
        }
    }
}

/// Information about a WIT function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Parameters as (name, type) pairs
    pub params: Vec<(String, String)>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_wit_loader() {
        // This test requires actual WIT files
        // It's here as a placeholder for when we have test data
    }
}
