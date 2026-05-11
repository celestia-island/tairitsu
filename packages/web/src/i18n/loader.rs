//! # I18n Loader
//!
//! Load and parse TOML language files.
//!
//! Supports two modes:
//! - **Typed**: `load_toml()` → `I18nKeys` (backward-compatible, requires exact struct match)
//! - **Flat**: `load_toml_flat()` → `HashMap<String, String>` (dot-path keys, used by `t!()` macro)

use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::i18n::keys::I18nKeys;

/// Load TOML content into the typed `I18nKeys` struct.
///
/// The TOML must exactly match the `I18nKeys` schema.
pub fn load_toml(toml_content: &str) -> Result<I18nKeys> {
    toml::from_str(toml_content).context("Failed to parse TOML i18n file")
}

/// Load TOML content into the typed `I18nKeys` struct (static version).
pub fn load_toml_static(toml_content: &'static str) -> Result<I18nKeys> {
    toml::from_str(toml_content).context("Failed to parse TOML i18n file")
}

/// Flatten a TOML table into a dot-path `HashMap<String, String>`.
///
/// Nested TOML tables become dot-separated keys:
///
/// ```toml
/// [common.button]
/// submit = "Submit"
/// cancel = "Cancel"
/// ```
///
/// Becomes: `{"common.button.submit": "Submit", "common.button.cancel": "Cancel"}`
pub fn load_toml_flat(toml_content: &str) -> Result<HashMap<String, String>> {
    let value: toml::Value =
        toml::from_str(toml_content).context("Failed to parse TOML i18n file")?;

    let mut map = HashMap::new();
    if let Some(table) = value.as_table() {
        flatten_toml_table(table, String::new(), &mut map);
    }
    Ok(map)
}

fn flatten_toml_table(
    table: &toml::map::Map<String, toml::Value>,
    prefix: String,
    output: &mut HashMap<String, String>,
) {
    for (key, value) in table {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };

        match value {
            toml::Value::String(s) => {
                output.insert(full_key, s.clone());
            }
            toml::Value::Table(nested) => {
                flatten_toml_table(nested, full_key, output);
            }
            _ => {
                output.insert(full_key, value.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_toml_flat_simple() {
        let toml = r#"
submit = "Submit"
cancel = "Cancel"
"#;
        let map = load_toml_flat(toml).unwrap();
        assert_eq!(map.get("submit"), Some(&"Submit".to_string()));
        assert_eq!(map.get("cancel"), Some(&"Cancel".to_string()));
    }

    #[test]
    fn test_load_toml_flat_nested() {
        let toml = r#"
[common.button]
submit = "Submit"
cancel = "Cancel"

[common.navigation]
home = "Home"

[page.home.hero]
title = "Welcome"
subtitle = "Subtitle"
"#;
        let map = load_toml_flat(toml).unwrap();
        assert_eq!(map.get("common.button.submit"), Some(&"Submit".to_string()));
        assert_eq!(map.get("common.button.cancel"), Some(&"Cancel".to_string()));
        assert_eq!(map.get("common.navigation.home"), Some(&"Home".to_string()));
        assert_eq!(
            map.get("page.home.hero.title"),
            Some(&"Welcome".to_string())
        );
        assert_eq!(
            map.get("page.home.hero.subtitle"),
            Some(&"Subtitle".to_string())
        );
    }

    #[test]
    fn test_load_toml_flat_empty() {
        let toml = "";
        let map = load_toml_flat(toml).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn test_load_toml_flat_mixed_values() {
        let toml = r#"
title = "Title"
count = 42
enabled = true
"#;
        let map = load_toml_flat(toml).unwrap();
        assert_eq!(map.get("title"), Some(&"Title".to_string()));
        assert_eq!(map.get("count"), Some(&"42".to_string()));
        assert_eq!(map.get("enabled"), Some(&"true".to_string()));
    }

    #[test]
    fn test_load_toml_flat_invalid_toml() {
        let toml = "invalid [toml";
        assert!(load_toml_flat(toml).is_err());
    }

    #[test]
    fn test_load_toml_typed() {
        let toml = r#"
[common.button]
submit = "Submit"
cancel = "Cancel"
confirm = "Confirm"
delete = "Delete"
edit = "Edit"
save = "Save"

[common.navigation]
home = "Home"
about = "About"
documentation = "Documentation"
components = "Components"
theme = "Theme"

[common.status]
loading = "Loading..."
success = "Success"
error = "Error"
warning = "Warning"

[theme]
light = "Light"
dark = "Dark"
auto = "Auto"

[page.home.hero]
title = "Welcome"
subtitle = "Subtitle"
description = "Description"
tagline = "Tagline"
explore = "Explore"

[page.home.features]
title = "Features"
description = "Features description"

[page.components]
title = "Components"
description = "Components description"

[page.documentation]
title = "Documentation"
description = "Documentation description"
getting_started = "Getting Started"
quick_start = "Quick Start"

[language]
english = "English"
chinese_simplified = "简体中文"
chinese_traditional = "繁體中文"
french = "Français"
russian = "Русский"
spanish = "Español"
arabic = "العربية"
japanese = "日本語"
korean = "한국어"

[sidebar.overview]
title = "Overview"

[sidebar.components]
title = "Components"

[sidebar.system]
title = "System"

[sidebar.demos]
title = "Demos"

[sidebar.items]
button = "Button"
form = "Form"
number_input = "Number Input"
search = "Search"
switch = "Switch"
feedback = "Feedback"
display = "Display"
avatar = "Avatar"
image = "Image"
tag = "Tag"
empty = "Empty"
comment = "Comment"
description_list = "Description List"
navigation = "Navigation"
collapsible = "Collapsible"
data = "Data"
table = "Table"
tree = "Tree"
pagination = "Pagination"
qrcode = "QR Code"
timeline = "Timeline"
cascader = "Cascader"
transfer = "Transfer"
media = "Media"
editor = "Editor"
visualization = "Visualization"
user_guide = "User Guide"
zoom_controls = "Zoom Controls"
form_demo = "Form Demo"
dashboard_demo = "Dashboard Demo"
video_demo = "Video Demo"
"#;
        let keys = load_toml(toml).unwrap();
        assert_eq!(keys.common.button.submit, "Submit");
        assert_eq!(keys.common.navigation.home, "Home");
        assert_eq!(keys.page.home.hero.title, "Welcome");
    }
}
