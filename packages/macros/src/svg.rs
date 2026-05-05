//! SVG embedding macro for compile-time SVG injection with XSS protection.
//!
//! This macro reads SVG content at compile time and creates a SafeSvg instance.
//! It supports inline SVG content, file paths, and resource index lookup by ID.
//!
//! # Features
//! - Compile-time SVG embedding
//! - XSS sanitization via SafeSvg
//! - Support for inline content, file paths, or resource ID lookup
//!
//! # Example
//! ```ignore
//! // Inline SVG content
//! let icon = svg! { r#"<path d="M12 2L2 22h20L12 2z"/>"# };
//!
//! // From file (relative to crate root)
//! let icon = svg! { file: "icons/sun.svg" };
//!
//! // From resource index by ID
//! let icon = svg! { id: "sun" };
//!
//! // Use with VElement
//! rsx! {
//!     svg {
//!         viewBox: "0 0 24 24",
//!         safe_svg: icon,
//!     }
//! }
//! ```

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Input for the svg! macro
pub struct SvgInput {
    /// SVG content source
    source: SvgSource,
}

/// Source of SVG content
enum SvgSource {
    /// Inline SVG content
    Inline(String),
    /// File path relative to crate root
    File(String),
    /// Resource ID (looked up in resource index)
    Id(String),
}

impl syn::parse::Parse for SvgInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check if it starts with a keyword or a string literal
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;

            if ident == "file" {
                input.parse::<syn::Token![:]>()?;
                let path: syn::LitStr = input.parse()?;
                Ok(SvgInput {
                    source: SvgSource::File(path.value()),
                })
            } else if ident == "id" {
                input.parse::<syn::Token![:]>()?;
                let id: syn::LitStr = input.parse()?;
                Ok(SvgInput {
                    source: SvgSource::Id(id.value()),
                })
            } else {
                Err(syn::Error::new(
                    ident.span(),
                    "expected `file:`, `id:`, or a string literal with SVG content",
                ))
            }
        } else if lookahead.peek(syn::LitStr) {
            // Inline SVG content
            let lit: syn::LitStr = input.parse()?;
            Ok(SvgInput {
                source: SvgSource::Inline(lit.value()),
            })
        } else {
            Err(lookahead.error())
        }
    }
}

/// Expands the svg! macro
pub fn expand_svg(input: TokenStream) -> TokenStream {
    let svg_input = syn::parse_macro_input!(input as SvgInput);

    let expanded = match svg_input.source {
        SvgSource::Inline(content) => expand_inline_svg(&content),
        SvgSource::File(path) => expand_file_svg(&path),
        SvgSource::Id(id) => expand_id_svg(&id),
    };

    TokenStream::from(expanded)
}

/// Expand inline SVG content
fn expand_inline_svg(content: &str) -> TokenStream2 {
    // Sanitize at compile time
    let sanitized = sanitize_svg(content);

    quote! {
        tairitsu::SafeSvg::from_static(#sanitized)
    }
}

/// Expand file-based SVG
fn expand_file_svg(path: &str) -> TokenStream2 {
    // Get the crate root directory
    let crate_root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let full_path = std::path::Path::new(&crate_root).join(path);

    // Read the file at compile time
    let content = match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(err) => {
            let error_msg = format!("Failed to read SVG file '{}': {}", path, err);
            return quote! {
                compile_error!(#error_msg)
            };
        }
    };

    // Sanitize at compile time
    let sanitized = sanitize_svg(&content);

    quote! {
        tairitsu::SafeSvg::from_static(#sanitized)
    }
}

/// Expand SVG by resource ID
fn expand_id_svg(id: &str) -> TokenStream2 {
    // Get the crate root and target directories
    let crate_root = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let crate_root_path = std::path::Path::new(&crate_root);

    // Try to find the SVG file by searching common locations
    let search_paths: Vec<std::path::PathBuf> = vec![
        crate_root_path.join("icons"),
        crate_root_path.join("src/icons"),
        crate_root_path.join("assets/icons"),
        crate_root_path.join("static/icons"),
        crate_root_path.join("resources/svg"),
        crate_root_path.to_path_buf(),
    ];

    // Try to find the file by ID
    for search_path in search_paths {
        let svg_path = search_path.join(format!("{}.svg", id));
        if svg_path.exists() {
            match std::fs::read_to_string(&svg_path) {
                Ok(content) => {
                    let sanitized = sanitize_svg(&content);
                    return quote! {
                        tairitsu::SafeSvg::from_static(#sanitized)
                    };
                }
                Err(err) => {
                    let error_msg =
                        format!("Failed to read SVG file '{}': {}", svg_path.display(), err);
                    return quote! {
                        compile_error!(#error_msg)
                    };
                }
            }
        }
    }

    // Try loading from resource index
    let target_dir = crate_root_path
        .parent()
        .map(|p| p.join("target"))
        .unwrap_or_else(|| std::path::PathBuf::from("target"));

    let index_path = target_dir.join("tairitsu/resources/index.json");

    if index_path.exists()
        && let Ok(index_content) = std::fs::read_to_string(&index_path)
        && let Ok(index) = serde_json::from_str::<ResourceIndexJson>(&index_content)
    {
        // Find SVG by ID
        for svg_entry in index.svg {
            if svg_entry.id == id {
                // Found by ID, read the source file
                let svg_path = crate_root_path.join(&svg_entry.source);
                match std::fs::read_to_string(&svg_path) {
                    Ok(content) => {
                        let sanitized = sanitize_svg(&content);
                        return quote! {
                            tairitsu::SafeSvg::from_static(#sanitized)
                        };
                    }
                    Err(err) => {
                        let error_msg = format!(
                            "Failed to read SVG file '{}' (indexed as '{}'): {}",
                            svg_entry.source, id, err
                        );
                        return quote! {
                            compile_error!(#error_msg)
                        };
                    }
                }
            }
        }
    }

    // Not found
    let error_msg = format!(
        "SVG with id '{}' not found. Searched in: icons/, src/icons/, assets/icons/, and resource index.",
        id
    );
    quote! {
        compile_error!(#error_msg)
    }
}

/// Resource index JSON structure for parsing
#[derive(Debug, serde::Deserialize)]
struct ResourceIndexJson {
    svg: Vec<SvgResourceJson>,
}

#[derive(Debug, serde::Deserialize)]
struct SvgResourceJson {
    id: String,
    source: String,
}

/// Sanitize SVG content at compile time
///
/// This performs the same sanitization as SafeSvg::new(), but at compile time
/// so the resulting binary only contains sanitized content.
fn sanitize_svg(content: &str) -> String {
    let mut result = content.to_string();

    // Remove script tags (including malformed ones)
    let script_pattern = regex::Regex::new(r"<script[^>]*>.*?</script>").unwrap();
    result = script_pattern.replace_all(&result, "").to_string();

    // Also handle malformed script tags without closing
    let script_open_pattern = regex::Regex::new(r"<script[^>]*>").unwrap();
    result = script_open_pattern.replace_all(&result, "").to_string();

    // Remove event handlers with double quotes (onclick="...", onload="...", etc.)
    let event_dq_pattern = regex::Regex::new(r#"\s+on\w+\s*=\s*"[^"]*""#).unwrap();
    result = event_dq_pattern.replace_all(&result, "").to_string();

    // Remove event handlers with single quotes
    let event_sq_pattern = regex::Regex::new(r#"\s+on\w+\s*=\s*'[^']*'"#).unwrap();
    result = event_sq_pattern.replace_all(&result, "").to_string();

    // Remove event handlers without quotes
    let event_unquoted_pattern = regex::Regex::new(r#"\s+on\w+\s*=\s*[^\s>]+"#).unwrap();
    result = event_unquoted_pattern.replace_all(&result, "").to_string();

    // Remove javascript: URLs
    let js_url_pattern = regex::Regex::new(r#"javascript\s*:"#).unwrap();
    result = js_url_pattern.replace_all(&result, "blocked:").to_string();

    // Remove dangerous data: URLs - replace data: URLs that are not images
    // Simple approach: block data:text/html, data:application, etc.
    let dangerous_data_pattern =
        regex::Regex::new(r#"data\s*:\s*(text/html|application|text/javascript)[^,]*,"#).unwrap();
    result = dangerous_data_pattern
        .replace_all(&result, "blocked:")
        .to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_removes_script() {
        let input = r#"<svg><script>alert('xss')</script><path d="M0 0"/></svg>"#;
        let result = sanitize_svg(input);
        assert!(!result.contains("<script"));
        assert!(result.contains("<path"));
    }

    #[test]
    fn test_sanitize_removes_event_handlers() {
        let input = r#"<svg onclick="alert('xss')"><path d="M0 0"/></svg>"#;
        let result = sanitize_svg(input);
        assert!(!result.contains("onclick"));
        assert!(result.contains("<path"));
    }

    #[test]
    fn test_sanitize_removes_javascript_url() {
        let input = r#"<a xlink:href="javascript:alert('xss')">link</a>"#;
        let result = sanitize_svg(input);
        assert!(!result.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_preserves_safe_content() {
        let input = r#"<path d="M12 2L2 22h20L12 2z" fill="currentColor"/>"#;
        let result = sanitize_svg(input);
        assert!(result.contains("M12 2L2 22h20L12 2z"));
        assert!(result.contains("fill"));
    }

    #[test]
    fn test_sanitize_preserves_fragment_reference() {
        let input = "<use xlink:href=\"#my-symbol\"/>";
        let result = sanitize_svg(input);
        assert!(result.contains("#my-symbol"));
    }
}
