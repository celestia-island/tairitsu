//! Icon metadata parsing for Cargo.toml and icon libraries.
//!
//! Handles parsing of `[package.metadata.tairitsu.icons]` configuration
//! and MDI meta.json format.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::IconStyle;

/// Icons configuration from Cargo.toml
#[derive(Debug, Clone, Deserialize, Default)]
pub struct IconsConfig {
    /// Icon source (mdi, lucide, custom)
    #[serde(default)]
    pub source: Option<String>,

    /// Specific icon names to include
    #[serde(default)]
    pub icons: Vec<String>,

    /// Tags to include (all icons with these tags)
    #[serde(default)]
    pub tags: Vec<String>,

    /// Style variants to include
    #[serde(default)]
    pub styles: Vec<String>,

    /// Output path for generated code
    #[serde(default)]
    pub output: Option<String>,

    /// MDI version to use (optional, defaults to latest)
    #[serde(default)]
    pub version: Option<String>,

    /// Custom icon directory (for custom source)
    #[serde(default)]
    pub custom_dir: Option<String>,
}

/// MDI meta.json single icon entry
///
/// Represents a single icon in the MDI meta.json format.
/// The MDI meta.json is an array of these objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdiIconMeta {
    /// Icon name (kebab-case, e.g., "account-circle")
    pub name: String,

    /// Aliases for this icon
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Tags/categories for searching
    #[serde(default)]
    pub tags: Vec<String>,

    /// Author of the icon
    #[serde(default)]
    pub author: String,

    /// Version when the icon was added
    #[serde(default)]
    pub version: String,

    /// Codepoint in the icon font
    #[serde(default)]
    pub codepoint: String,

    /// Whether this icon is deprecated
    #[serde(default)]
    pub deprecated: bool,

    /// SVG path data (d attribute value)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl MdiIconMeta {
    /// Convert the icon name to a valid Rust identifier
    pub fn to_rust_ident(&self) -> String {
        let mut ident = self.name.replace('-', "_");
        // Ensure it doesn't start with a number
        if ident
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            ident = format!("Icon{}", ident);
        }
        ident
    }

    /// Generate SVG content from path data
    pub fn to_svg(&self) -> String {
        if let Some(ref path) = self.path {
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="{}"/></svg>"#,
                path
            )
        } else {
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><!-- No path data for {} --></svg>"#,
                self.name
            )
        }
    }
}

/// MDI meta.json root structure
///
/// The MDI meta.json is a JSON array of icon metadata objects.
/// This struct wraps the parsed data with additional metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MdiMetadata {
    /// Parsed icon entries indexed by name
    #[serde(skip)]
    pub icons: HashMap<String, MdiIconMeta>,

    /// Version of the MDI package
    pub version: String,

    /// Total icon count
    pub count: usize,
}

impl MdiMetadata {
    /// Parse MDI meta.json content
    ///
    /// The meta.json format is an array of icon objects:
    /// ```json
    /// [
    ///   {"name": "account", "tags": ["Account"], "aliases": ["user"], ...},
    ///   ...
    /// ]
    /// ```
    pub fn parse(json: &str) -> crate::Result<Self> {
        let entries: Vec<serde_json::Value> = serde_json::from_str(json)?;

        let mut icons = HashMap::new();

        for entry in entries {
            if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                let icon_meta = MdiIconMeta {
                    name: name.to_string(),
                    aliases: entry
                        .get("aliases")
                        .and_then(|a| a.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    tags: entry
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    author: entry
                        .get("author")
                        .and_then(|a| a.as_str())
                        .unwrap_or("")
                        .to_string(),
                    version: entry
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    codepoint: entry
                        .get("codepoint")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string(),
                    deprecated: entry
                        .get("deprecated")
                        .and_then(|d| d.as_bool())
                        .unwrap_or(false),
                    path: None, // Path is loaded separately from SVG files
                };
                icons.insert(name.to_string(), icon_meta);
            }
        }

        let count = icons.len();

        Ok(Self {
            icons,
            version: String::new(),
            count,
        })
    }

    /// Get an icon by name
    pub fn get(&self, name: &str) -> Option<&MdiIconMeta> {
        self.icons.get(name)
    }

    /// Get all available tags
    pub fn available_tags(&self) -> Vec<String> {
        let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();

        for icon in self.icons.values() {
            for tag in &icon.tags {
                tags.insert(tag.clone());
            }
        }

        let mut tags: Vec<_> = tags.into_iter().collect();
        tags.sort();
        tags
    }

    /// Filter icons by names and tags
    pub fn filter(&self, names: &[String], tags: &[String]) -> Vec<&MdiIconMeta> {
        if names.is_empty() && tags.is_empty() {
            return self.icons.values().filter(|i| !i.deprecated).collect();
        }

        self.icons
            .values()
            .filter(|icon| {
                if icon.deprecated {
                    return false;
                }

                let name_matches = names.iter().any(|n| {
                    icon.name == *n
                        || icon.name.replace('-', "_") == n.replace('-', "_")
                        || icon.aliases.iter().any(|a| a == n)
                });

                let tag_matches = tags.iter().any(|t| {
                    icon.tags
                        .iter()
                        .any(|icon_tag| icon_tag.to_lowercase() == t.to_lowercase())
                });

                name_matches || tag_matches
            })
            .collect()
    }

    /// Search icons by query string
    pub fn search(&self, query: &str) -> Vec<&MdiIconMeta> {
        let query_lower = query.to_lowercase();

        self.icons
            .values()
            .filter(|icon| {
                icon.name.to_lowercase().contains(&query_lower)
                    || icon
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || icon
                        .aliases
                        .iter()
                        .any(|a| a.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

/// Parse icon metadata from JSON (MDI format)
pub fn parse_mdi_metadata(json: &str) -> crate::Result<MdiMetadata> {
    MdiMetadata::parse(json)
}

impl IconsConfig {
    /// Convert to IconConfig
    pub fn to_icon_config(&self) -> super::IconConfig {
        let source = self
            .source
            .as_ref()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default();

        let styles = if self.styles.is_empty() {
            vec![IconStyle::Filled]
        } else {
            self.styles
                .iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "filled" | "solid" => Some(IconStyle::Filled),
                    "outline" | "outlined" => Some(IconStyle::Outline),
                    _ => None,
                })
                .collect()
        };

        let output = self
            .output
            .as_ref()
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("src/generated/icons.rs"));

        super::IconConfig {
            source,
            names: self.icons.clone(),
            tags: self.tags.clone(),
            styles,
            output,
        }
    }

    /// Check if icons are configured
    pub fn is_configured(&self) -> bool {
        !self.icons.is_empty() || !self.tags.is_empty() || self.source.is_some()
    }
}

/// Single icon entry from MDI metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconEntry {
    /// Icon name (kebab-case)
    pub name: String,

    /// Aliases for this icon
    #[serde(default)]
    pub aliases: Vec<String>,

    /// Tags/categories
    #[serde(default)]
    pub tags: Vec<String>,

    /// Author
    #[serde(default)]
    pub author: Option<String>,

    /// Version when added
    #[serde(default)]
    pub version: Option<String>,

    /// Deprecated (if true, this icon is deprecated)
    #[serde(default)]
    pub deprecated: bool,

    /// SVG path data (filled in after fetching)
    #[serde(skip)]
    pub svg_path: Option<String>,
}

impl IconEntry {
    /// Convert name to valid Rust identifier
    pub fn to_rust_ident(&self) -> String {
        let mut ident = self.name.replace('-', "_");
        // Ensure it doesn't start with a number
        if ident
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            ident = format!("Icon{}", ident);
        }
        ident
    }
}

/// Icon metadata collection (from MDI meta.json)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IconMetadata {
    /// Icon entries by name
    pub icons: HashMap<String, IconEntry>,

    /// Source version
    pub version: String,

    /// Total count
    pub count: usize,
}

impl IconMetadata {
    /// Parse from MDI meta.json format
    pub fn parse_mdi_json(json: &str) -> crate::Result<Self> {
        // MDI meta.json is an array of icon objects
        let entries: Vec<serde_json::Value> = serde_json::from_str(json)?;

        let mut icons = HashMap::new();

        for entry in entries {
            if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                let icon_entry = IconEntry {
                    name: name.to_string(),
                    aliases: entry
                        .get("aliases")
                        .and_then(|a| a.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    tags: entry
                        .get("tags")
                        .and_then(|t| t.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default(),
                    author: entry
                        .get("author")
                        .and_then(|a| a.as_str())
                        .map(String::from),
                    version: entry
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    deprecated: entry
                        .get("deprecated")
                        .and_then(|d| d.as_bool())
                        .unwrap_or(false),
                    svg_path: None,
                };
                icons.insert(name.to_string(), icon_entry);
            }
        }

        let count = icons.len();

        Ok(Self {
            icons,
            version: String::new(),
            count,
        })
    }

    /// Filter icons by names and tags
    pub fn filter_icons(&self, names: &[String], tags: &[String]) -> Vec<&IconEntry> {
        if names.is_empty() && tags.is_empty() {
            // Return all non-deprecated icons
            return self.icons.values().filter(|i| !i.deprecated).collect();
        }

        self.icons
            .values()
            .filter(|icon| {
                if icon.deprecated {
                    return false;
                }

                // Check if name matches
                let name_matches = names.iter().any(|n| {
                    icon.name == *n
                        || icon.name.replace('-', "_") == n.replace('-', "_")
                        || icon.aliases.iter().any(|a| a == n)
                });

                // Check if tag matches
                let tag_matches = tags.iter().any(|t| {
                    icon.tags
                        .iter()
                        .any(|icon_tag| icon_tag.to_lowercase() == t.to_lowercase())
                });

                name_matches || tag_matches
            })
            .collect()
    }

    /// Get icon by name
    pub fn get(&self, name: &str) -> Option<&IconEntry> {
        self.icons.get(name)
    }

    /// List all available tags
    pub fn available_tags(&self) -> Vec<String> {
        let mut tags: std::collections::HashSet<String> = std::collections::HashSet::new();

        for icon in self.icons.values() {
            for tag in &icon.tags {
                tags.insert(tag.clone());
            }
        }

        let mut tags: Vec<_> = tags.into_iter().collect();
        tags.sort();
        tags
    }

    /// Search icons by query
    pub fn search(&self, query: &str) -> Vec<&IconEntry> {
        let query_lower = query.to_lowercase();

        self.icons
            .values()
            .filter(|icon| {
                icon.name.to_lowercase().contains(&query_lower)
                    || icon
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
                    || icon
                        .aliases
                        .iter()
                        .any(|a| a.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
}

/// Parse icons configuration from Cargo.toml metadata
pub fn parse_icons_config(manifest: &toml::Value) -> crate::Result<IconsConfig> {
    let icons_config = manifest
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("tairitsu"))
        .and_then(|t| t.get("icons"));

    match icons_config {
        Some(value) => {
            let config: IconsConfig = value.clone().try_into().map_err(|e| {
                crate::TairitsuPackagerError::InvalidConfig(format!(
                    "Invalid icons configuration: {}",
                    e
                ))
            })?;
            Ok(config)
        }
        None => Ok(IconsConfig::default()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mdi_json() {
        let json = r#"[
            {"name": "account", "tags": ["Account"], "aliases": ["user"]},
            {"name": "sun", "tags": ["Nature", "Weather"]}
        ]"#;

        let meta = IconMetadata::parse_mdi_json(json).unwrap();
        assert_eq!(meta.count, 2);
        assert!(meta.get("account").is_some());
        assert!(meta.get("sun").is_some());
    }

    #[test]
    fn test_filter_by_name() {
        let json = r#"[
            {"name": "account", "tags": ["Account"]},
            {"name": "sun", "tags": ["Nature"]},
            {"name": "moon", "tags": ["Nature"]}
        ]"#;

        let meta = IconMetadata::parse_mdi_json(json).unwrap();
        let filtered = meta.filter_icons(&["sun".to_string()], &[]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "sun");
    }

    #[test]
    fn test_filter_by_tag() {
        let json = r#"[
            {"name": "account", "tags": ["Account"]},
            {"name": "sun", "tags": ["Nature"]},
            {"name": "moon", "tags": ["Nature"]}
        ]"#;

        let meta = IconMetadata::parse_mdi_json(json).unwrap();
        let filtered = meta.filter_icons(&[], &["Nature".to_string()]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_to_rust_ident() {
        let entry = IconEntry {
            name: "moon-waning-crescent".to_string(),
            aliases: vec![],
            tags: vec![],
            author: None,
            version: None,
            deprecated: false,
            svg_path: None,
        };
        assert_eq!(entry.to_rust_ident(), "moon_waning_crescent");
    }

    #[test]
    fn test_mdi_metadata_parse() {
        let json = r#"[
            {"name": "account-circle", "tags": ["Account"], "aliases": ["user-circle"], "author": "Google", "version": "1.5.54"},
            {"name": "sun", "tags": ["Nature", "Weather"], "author": "Templarian"}
        ]"#;

        let meta = parse_mdi_metadata(json).unwrap();
        assert_eq!(meta.count, 2);
        assert!(meta.get("account-circle").is_some());
        assert!(meta.get("sun").is_some());
    }

    #[test]
    fn test_mdi_icon_meta_to_rust_ident() {
        let icon = MdiIconMeta {
            name: "account-circle".to_string(),
            aliases: vec!["user-circle".to_string()],
            tags: vec!["Account".to_string()],
            author: "Google".to_string(),
            version: "1.5.54".to_string(),
            codepoint: "F0019".to_string(),
            deprecated: false,
            path: Some(
                "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z".to_string(),
            ),
        };
        assert_eq!(icon.to_rust_ident(), "account_circle");
    }

    #[test]
    fn test_mdi_icon_meta_to_svg() {
        let icon = MdiIconMeta {
            name: "test".to_string(),
            aliases: vec![],
            tags: vec![],
            author: String::new(),
            version: String::new(),
            codepoint: String::new(),
            deprecated: false,
            path: Some("M12 2L2 22h20L12 2z".to_string()),
        };

        let svg = icon.to_svg();
        assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
        assert!(svg.contains("viewBox=\"0 0 24 24\""));
        assert!(svg.contains("M12 2L2 22h20L12 2z"));
    }

    #[test]
    fn test_mdi_icon_meta_to_svg_no_path() {
        let icon = MdiIconMeta {
            name: "no-path".to_string(),
            aliases: vec![],
            tags: vec![],
            author: String::new(),
            version: String::new(),
            codepoint: String::new(),
            deprecated: false,
            path: None,
        };

        let svg = icon.to_svg();
        assert!(svg.contains("<!-- No path data for no-path -->"));
    }

    #[test]
    fn test_mdi_metadata_filter() {
        let json = r#"[
            {"name": "account", "tags": ["Account"]},
            {"name": "sun", "tags": ["Nature"]},
            {"name": "moon", "tags": ["Nature"]}
        ]"#;

        let meta = MdiMetadata::parse(json).unwrap();

        // Filter by name
        let filtered = meta.filter(&["sun".to_string()], &[]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "sun");

        // Filter by tag
        let filtered = meta.filter(&[], &["Nature".to_string()]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_mdi_metadata_search() {
        let json = r#"[
            {"name": "account-circle", "tags": ["Account"], "aliases": ["user"]},
            {"name": "account-outline", "tags": ["Account"]},
            {"name": "sun", "tags": ["Nature"]}
        ]"#;

        let meta = MdiMetadata::parse(json).unwrap();
        let results = meta.search("account");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_mdi_metadata_available_tags() {
        let json = r#"[
            {"name": "account", "tags": ["Account", "User"]},
            {"name": "sun", "tags": ["Nature", "Weather"]},
            {"name": "moon", "tags": ["Nature"]}
        ]"#;

        let meta = MdiMetadata::parse(json).unwrap();
        let tags = meta.available_tags();
        assert_eq!(tags.len(), 4);
        assert!(tags.contains(&"Account".to_string()));
        assert!(tags.contains(&"Nature".to_string()));
    }
}
