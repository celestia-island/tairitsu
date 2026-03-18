//! SCSS macro for compile-time CSS generation with class name hashing.
//!
//! This macro compiles SCSS content to CSS at compile time and generates
//! hashed class names for CSS Modules-style scoping.
//!
//! # Features
//! - Full SCSS syntax support via grass compiler
//! - Automatic class name hashing (CSS Modules style)
//! - Scope-based isolation
//! - Support for inline content, file paths, or resource ID lookup
//!
//! # Example
//! ```ignore
//! // Inline SCSS content
//! let (css, class_map) = scss! {
//!     .button {
//!         background: var(--primary);
//!         color: white;
//!     }
//! };
//!
//! // From file (relative to crate root)
//! let (css, class_map) = scss! { file: "styles/main.scss" };
//!
//! // With scope for isolation
//! let (css, class_map) = scss! {
//!     .container {
//!         width: 100%;
//!     },
//!     scope: "MyComponent"
//! };
//!
//! // Use hashed class names
//! let button_class = class_map.get("button").unwrap();
//! ```

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use sha2::{Digest, Sha256};

/// Source of SCSS content
pub enum ScssSource {
    /// Inline SCSS content
    Inline(String),
    /// File path relative to crate root
    File(String),
}

/// Input for the scss! macro
pub struct ScssInput {
    /// SCSS content source
    source: ScssSource,
    /// Optional scope for class name isolation
    scope: Option<String>,
}

impl syn::parse::Parse for ScssInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check if it starts with a keyword or a string literal
        let lookahead = input.lookahead1();

        let (source, scope) = if lookahead.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;

            if ident == "file" {
                // file: "path/to/file.scss"
                input.parse::<syn::Token![:]>()?;
                let path: syn::LitStr = input.parse()?;
                let scope = parse_scope(input)?;
                (ScssSource::File(path.value()), scope)
            } else if ident == "scope" {
                // Backward compatible: starting with scope
                input.parse::<syn::Token![:]>()?;
                let scope_lit: syn::LitStr = input.parse()?;
                let scope = Some(scope_lit.value());

                // Then expect the content
                input.parse::<syn::Token![,]>()?;
                let lit: syn::LitStr = input.parse()?;
                (ScssSource::Inline(lit.value()), scope)
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "expected `file:`, `scope:`, or SCSS content starting with a class selector",
                ));
            }
        } else if lookahead.peek(syn::LitStr) {
            // String literal - could be inline SCSS or file path
            let lit: syn::LitStr = input.parse()?;
            let content = lit.value();

            // Check if this looks like inline SCSS (contains { or .)
            let is_inline_scss = content.contains('{') || content.contains('.');

            if is_inline_scss {
                let scope = parse_scope(input)?;
                (ScssSource::Inline(content), scope)
            } else {
                // Treat as file path
                let scope = parse_scope(input)?;
                (ScssSource::File(content), scope)
            }
        } else {
            // Try to parse as raw SCSS (class selectors like .button)
            // This handles the case where the input starts with `.`
            let tts: proc_macro2::TokenStream = input.parse()?;
            let scss_content = tts.to_string();

            // Simple heuristic: if it looks like SCSS, treat it as inline
            if scss_content.contains('{') || scss_content.starts_with('.') {
                let scope = parse_scope(input)?;
                (ScssSource::Inline(scss_content), scope)
            } else {
                return Err(lookahead.error());
            }
        };

        Ok(ScssInput { source, scope })
    }
}

/// Parse optional scope parameter
fn parse_scope(input: syn::parse::ParseStream) -> syn::Result<Option<String>> {
    let mut scope = None;

    while !input.is_empty() {
        input.parse::<syn::Token![,]>()?;
        if input.is_empty() {
            break;
        }

        let ident: syn::Ident = input.parse()?;
        if ident == "scope" {
            input.parse::<syn::Token![:]>()?;
            let scope_lit: syn::LitStr = input.parse()?;
            scope = Some(scope_lit.value());
        }
    }

    Ok(scope)
}

/// Expands the scss! macro
pub fn expand_scss(input: TokenStream) -> TokenStream {
    let scss_input = syn::parse_macro_input!(input as ScssInput);

    let expanded = match scss_input.source {
        ScssSource::Inline(content) => expand_inline_scss(&content, scss_input.scope.as_deref()),
        ScssSource::File(path) => expand_file_scss(&path, scss_input.scope.as_deref()),
    };

    TokenStream::from(expanded)
}

/// Expand inline SCSS content
fn expand_inline_scss(content: &str, scope: Option<&str>) -> TokenStream2 {
    let (css, class_map) = compile_scss_with_hashing(content, scope);
    generate_output(css, class_map)
}

/// Expand file-based SCSS
fn expand_file_scss(path: &str, scope: Option<&str>) -> TokenStream2 {
    // Get the crate root directory
    let crate_root = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");

    let full_path = std::path::Path::new(&crate_root).join(path);

    // Read the file at compile time
    let content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(err) => {
            let error_msg = format!("Failed to read SCSS file '{}': {}", path, err);
            return quote! {
                compile_error!(#error_msg)
            };
        }
    };

    let (css, class_map) = compile_scss_with_hashing(&content, scope);
    generate_output(css, class_map)
}

/// Generate the output token stream
fn generate_output(css: String, class_map: HashMap<String, String>) -> TokenStream2 {
    let map_entries: Vec<_> = class_map
        .into_iter()
        .map(|(original, hashed)| {
            let original_str = original.as_str();
            let hashed_str = hashed.as_str();
            quote! { (#original_str, #hashed_str) }
        })
        .collect();

    quote! {
        {
            let css = #css;
            let class_map = std::collections::HashMap::from([
                #(#map_entries),*
            ]);
            (css, class_map)
        }
    }
}

fn compile_scss_with_hashing(scss: &str, scope: Option<&str>) -> (String, HashMap<String, String>) {
    let mut class_map = HashMap::new();

    let hash_input = match scope {
        Some(s) => format!("{}:{}", s, scss),
        None => scss.to_string(),
    };

    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let hash = hasher.finalize();
    let hash_str = hex::encode(&hash[..6]);

    let processed_scss = process_class_names(scss, &hash_str, &mut class_map);

    let css = match grass::from_string(&processed_scss, &grass::Options::default()) {
        Ok(css) => css,
        Err(e) => {
            eprintln!("SCSS compilation failed: {}", e);
            format!("/* CSS generation failed: {} */", e)
        }
    };

    (css, class_map)
}

fn process_class_names(scss: &str, hash: &str, class_map: &mut HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut current_class = String::new();
    let mut in_class_context = false;

    for ch in scss.chars() {
        if ch == '.' && !in_class_context {
            in_class_context = true;
            current_class.clear();
        } else if in_class_context {
            if ch.is_whitespace() || ch == '{' {
                if !current_class.is_empty() {
                    let hashed_class = format!("{}_{}", current_class, hash);
                    class_map.insert(current_class.clone(), hashed_class.clone());
                    result.push_str(&hashed_class);
                }
                result.push(ch);
                in_class_context = false;
            } else {
                current_class.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_basic_scss() {
        let scss = r#"
            .button {
                background: blue;
                color: white;
            }
        "#;

        let (css, class_map) = compile_scss_with_hashing(scss, None);

        assert!(css.contains("background:"));
        assert!(!class_map.is_empty());
    }

    #[test]
    fn test_nested_scss() {
        let scss = r#"
            .container {
                width: 100%;

                .item {
                    padding: 8px;
                }
            }
        "#;

        let (css, class_map) = compile_scss_with_hashing(scss, None);

        assert!(css.contains("width:"));
        assert!(class_map.contains_key("container"));
    }

    #[test]
    fn test_scope_isolation() {
        let scss = r#"
            .button {
                color: red;
            }
        "#;

        let (_css1, map1) = compile_scss_with_hashing(scss, Some("component1"));
        let (_css2, map2) = compile_scss_with_hashing(scss, Some("component2"));

        assert_ne!(map1.get("button").unwrap(), map2.get("button").unwrap());
    }
}
