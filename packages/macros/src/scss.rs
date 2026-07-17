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
    /// Skip CSS Modules hashing (for global CSS like site layouts).
    no_hash: bool,
}

impl syn::parse::Parse for ScssInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        let (source, scope, no_hash) = if lookahead.peek(syn::Ident) {
            let ident: syn::Ident = input.parse()?;

            if ident == "file" {
                input.parse::<syn::Token![:]>()?;
                let path: syn::LitStr = input.parse()?;
                let (scope, no_hash) = parse_scope(input)?;
                (ScssSource::File(path.value()), scope, no_hash)
            } else if ident == "scope" {
                // Backward compatible: `scope: "...", "<inline scss>"`.
                input.parse::<syn::Token![:]>()?;
                let scope_lit: syn::LitStr = input.parse()?;
                input.parse::<syn::Token![,]>()?;
                let lit: syn::LitStr = input.parse()?;
                (
                    ScssSource::Inline(lit.value()),
                    Some(scope_lit.value()),
                    false,
                )
            } else {
                return Err(syn::Error::new(ident.span(), "expected `file:`, `scope:`"));
            }
        } else if lookahead.peek(syn::LitStr) {
            let lit: syn::LitStr = input.parse()?;
            let content = lit.value();
            let is_inline_scss = content.contains('{') || content.contains('.');
            let (scope, no_hash) = parse_scope(input)?;
            let source = if is_inline_scss {
                ScssSource::Inline(content)
            } else {
                ScssSource::File(content)
            };
            (source, scope, no_hash)
        } else {
            let tts: proc_macro2::TokenStream = input.parse()?;
            let scss_content = tts.to_string();
            if scss_content.contains('{') || scss_content.starts_with('.') {
                let (scope, no_hash) = parse_scope(input)?;
                (ScssSource::Inline(scss_content), scope, no_hash)
            } else {
                return Err(lookahead.error());
            }
        };

        Ok(ScssInput {
            source,
            scope,
            no_hash,
        })
    }
}

/// Parse optional trailing `, scope: "..."` and `, no_hash`. Unknown options
/// are rejected outright — a silently ignored `no_hash` is how unhashed
/// global CSS used to end up hashed anyway.
fn parse_scope(input: syn::parse::ParseStream) -> syn::Result<(Option<String>, bool)> {
    let mut scope = None;
    let mut no_hash = false;

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
        } else if ident == "no_hash" {
            no_hash = true;
        } else {
            return Err(syn::Error::new(
                ident.span(),
                "unknown scss! option; expected `scope:` or `no_hash`",
            ));
        }
    }

    Ok((scope, no_hash))
}

/// Expands the scss! macro
pub fn expand_scss(input: TokenStream) -> TokenStream {
    let scss_input = syn::parse_macro_input!(input as ScssInput);

    let expanded = match scss_input.source {
        ScssSource::Inline(content) => {
            expand_inline_scss(&content, scss_input.scope.as_deref(), scss_input.no_hash)
        }
        ScssSource::File(path) => {
            expand_file_scss(&path, scss_input.scope.as_deref(), scss_input.no_hash)
        }
    };

    TokenStream::from(expanded)
}

/// Expand inline SCSS content
fn expand_inline_scss(content: &str, scope: Option<&str>, no_hash: bool) -> TokenStream2 {
    let (css, class_map) = compile_scss_with_hashing(content, scope, no_hash);
    generate_output(css, class_map)
}

/// Expand file-based SCSS
fn expand_file_scss(path: &str, scope: Option<&str>, no_hash: bool) -> TokenStream2 {
    let crate_root = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(_) => {
            return quote! { compile_error!("CARGO_MANIFEST_DIR not set — cannot resolve SCSS file path") }
        }
    };

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

    // grass compiles from a string, so relative `@use`/`@import` paths have no
    // base directory by default and fail with "can't find stylesheet". Offer
    // the SCSS file's own directory (SCSS-relative semantics) and the crate
    // root as load paths.
    let mut load_paths: Vec<std::path::PathBuf> = Vec::new();
    if let Some(parent) = full_path.parent() {
        load_paths.push(parent.to_path_buf());
    }
    load_paths.push(std::path::PathBuf::from(&crate_root));

    let (css, class_map) =
        compile_scss_inner(&content, scope, no_hash, &load_paths, Some(&full_path));
    wrap_with_dep_tracking(&full_path, generate_output(css, class_map))
}

/// Re-emit the entry file through `include_str!` so rustc lists it in the
/// crate's dep-info: editing the SCSS then re-triggers macro expansion.
/// Without this, cargo's fingerprint sees no `.rs` change and silently
/// reuses stale CSS from a previous compile. (Partials pulled in via
/// `@use`/`@import` are still only re-read when the entry file changes.)
fn wrap_with_dep_tracking(full_path: &std::path::Path, output: TokenStream2) -> TokenStream2 {
    // Forward slashes: valid on every platform, no backslash escaping.
    let tracking_path = full_path.to_string_lossy().replace('\\', "/");
    quote! {
        {
            const _: &'static str = include_str!(#tracking_path);
            #output
        }
    }
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
            // Explicit types: with `no_hash` (or any class-less input) the map
            // is empty, and `HashMap::from([])` cannot infer K/V otherwise.
            let css: &'static str = #css;
            let class_map: std::collections::HashMap<&'static str, &'static str> =
                std::collections::HashMap::from([
                    #(#map_entries),*
                ]);
            (css, class_map)
        }
    }
}

fn compile_scss_with_hashing(
    scss: &str,
    scope: Option<&str>,
    no_hash: bool,
) -> (String, HashMap<String, String>) {
    compile_scss_inner(scss, scope, no_hash, &[], None)
}

fn compile_scss_inner(
    scss: &str,
    scope: Option<&str>,
    no_hash: bool,
    load_paths: &[std::path::PathBuf],
    source_path: Option<&std::path::Path>,
) -> (String, HashMap<String, String>) {
    let mut class_map = HashMap::new();

    let processed_scss = if no_hash {
        scss.to_string()
    } else {
        let hash_input = match scope {
            Some(s) => format!("{}:{}", s, scss),
            None => scss.to_string(),
        };

        let mut hasher = Sha256::new();
        hasher.update(hash_input.as_bytes());
        let hash = hasher.finalize();
        let hash_str = hex::encode(&hash[..6]);

        process_class_names(scss, &hash_str, &mut class_map)
    };

    let options = grass::Options::default().load_paths(load_paths);
    // For untouched (no_hash) file sources, let grass read the file itself so
    // relative `@use`/`@import` urls resolve against the file's own directory.
    // `from_string` gives them a synthetic `./stdin` base instead, and grass's
    // load-path fallback is skipped entirely for extension-qualified urls
    // (its `find_import` short-circuits with a `// todo: consider load paths`).
    let compiled = match source_path {
        Some(p) if no_hash => grass::from_path(p, &options),
        _ => grass::from_string(&processed_scss, &options),
    };
    let css = match compiled {
        Ok(css) => css,
        Err(e) => {
            eprintln!("SCSS compilation failed: {}", e);
            format!("/* CSS generation failed: {} */", e)
        }
    };

    (css, class_map)
}

/// Rewrite class selectors to their hashed names for CSS-Modules-style
/// scoping. Everything that is *not* a class selector must survive verbatim:
///
/// - A `.` only opens a class candidate when the next character can start a
///   CSS ident (letter, `_`, `-`); a digit means a decimal literal (`0.08`,
///   `.5rem`) and the dot is passed through untouched.
/// - `//` and `/* */` comments, `'…'` / `"…"` strings, `url(…)`, and
///   interpolation `#{…}` are scanned verbatim — dots inside them are never
///   class selectors.
/// - A candidate terminated by `(` is a sass built-in function call such as
///   `map.get(…)`, not a selector, so the name is emitted unhashed.
/// - The leading dot is preserved in the output (`.foo_hash`), and *any*
///   non-ident character terminates a candidate — so `.foo&.bar`,
///   `.foo:hover`, `.foo[x]`, and `:not(.foo)` all survive intact.
fn process_class_names(scss: &str, hash: &str, class_map: &mut HashMap<String, String>) -> String {
    fn flush(
        result: &mut String,
        current: &mut String,
        hash: &str,
        map: &mut HashMap<String, String>,
    ) {
        if !current.is_empty() {
            let hashed = format!("{}_{}", current, hash);
            map.insert(current.clone(), hashed.clone());
            result.push_str(&hashed);
            current.clear();
        }
    }

    let chars: Vec<char> = scss.chars().collect();
    let n = chars.len();
    let mut result = String::with_capacity(scss.len());
    let mut current_class = String::new();
    let mut in_class = false;
    let mut in_url = false;
    let mut in_string: Option<char> = None;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut interp_depth = 0usize;

    let mut i = 0;
    while i < n {
        let ch = chars[i];
        let next = chars.get(i + 1).copied();

        // ── verbatim states: comments, strings, url(...), #{...} ─────────
        if in_line_comment {
            result.push(ch);
            if ch == '\n' {
                in_line_comment = false;
            }
            i += 1;
            continue;
        }
        if in_block_comment {
            result.push(ch);
            if ch == '*' && next == Some('/') {
                result.push('/');
                i += 2;
                in_block_comment = false;
            } else {
                i += 1;
            }
            continue;
        }
        if let Some(quote) = in_string {
            result.push(ch);
            if ch == '\\' {
                if let Some(escaped) = next {
                    result.push(escaped);
                    i += 2;
                    continue;
                }
            }
            if ch == quote {
                in_string = None;
            }
            i += 1;
            continue;
        }
        if in_url {
            result.push(ch);
            if ch == ')' {
                in_url = false;
            }
            i += 1;
            continue;
        }
        if interp_depth > 0 {
            result.push(ch);
            match ch {
                '{' => interp_depth += 1,
                '}' => interp_depth -= 1,
                _ => {}
            }
            i += 1;
            continue;
        }

        // ── inside a class candidate: accumulate ident chars, else flush ──
        if in_class {
            if ch.is_alphanumeric() || ch == '-' || ch == '_' {
                current_class.push(ch);
                i += 1;
                continue;
            }
            if ch == '(' {
                // `name(` — a sass built-in call like `map.get(`, not a class.
                result.push_str(&current_class);
                current_class.clear();
                in_class = false;
                result.push(ch);
                i += 1;
                continue;
            }
            flush(&mut result, &mut current_class, hash, class_map);
            in_class = false;
            // Fall through: `ch` may itself open a new state (a chained `.bar`
            // in `.foo.bar`, a string, a comment, ...).
        }

        // ── state entry ───────────────────────────────────────────────────
        if ch == '/' && next == Some('/') {
            in_line_comment = true;
            result.push(ch);
        } else if ch == '/' && next == Some('*') {
            in_block_comment = true;
            result.push(ch);
        } else if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            result.push(ch);
        } else if ch == '#' && next == Some('{') {
            interp_depth = 1;
            result.push(ch);
        } else if (ch == 'u' || ch == 'U')
            && chars[i..]
                .iter()
                .take(4)
                .collect::<String>()
                .eq_ignore_ascii_case("url(")
        {
            in_url = true;
            result.push(ch);
        } else if ch == '.' {
            // A dot only opens a class candidate when the next char can start
            // a CSS ident; a digit means a decimal literal (`0.08`, `.5rem`).
            let starts_class = next
                .map(|c| c.is_alphabetic() || c == '_' || c == '-')
                .unwrap_or(false);
            result.push('.');
            if starts_class {
                in_class = true;
                current_class.clear();
            }
        } else {
            result.push(ch);
        }
        i += 1;
    }

    if in_class {
        flush(&mut result, &mut current_class, hash, class_map);
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

        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

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

        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

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

        let (_css1, map1) = compile_scss_with_hashing(scss, Some("component1"), false);
        let (_css2, map2) = compile_scss_with_hashing(scss, Some("component2"), false);

        assert_ne!(map1.get("button").unwrap(), map2.get("button").unwrap());
    }

    #[test]
    fn test_url_with_dots() {
        let scss = r#"
            .card {
                background-image: url(http://example.com/image.png);
            }
        "#;

        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

        assert!(css.contains("url("));
        assert!(class_map.contains_key("card"));
        assert!(!class_map.contains_key("png"));
    }

    #[test]
    fn test_chained_selectors() {
        let scss = r#"
            .foo.bar {
                color: red;
            }
        "#;

        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

        assert!(class_map.contains_key("foo"));
        assert!(class_map.contains_key("bar"));
        let foo_hashed = class_map.get("foo").unwrap();
        let bar_hashed = class_map.get("bar").unwrap();
        assert!(css.contains(foo_hashed));
        assert!(css.contains(bar_hashed));
    }

    #[test]
    fn test_no_hash_passes_source_through_verbatim() {
        let scss = r"
            $code-bg: #131e32;
            .button { background: rgba(122, 162, 247, 0.08); }
        ";

        let (css, class_map) = compile_scss_with_hashing(scss, None, true);

        assert!(class_map.is_empty());
        assert!(css.contains(".button"), "class kept verbatim, got: {css}");
        assert!(css.contains("0.08"), "decimal kept verbatim, got: {css}");
        assert!(!css.contains("CSS generation failed"), "got: {css}");
    }

    #[test]
    fn test_hashed_output_keeps_leading_dot() {
        let scss = ".button { color: red; }";
        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

        let hashed = class_map.get("button").unwrap();
        assert!(
            css.contains(&format!(".{hashed}")),
            "selector must keep its dot, got: {css}"
        );
    }

    #[test]
    fn test_decimal_literals_are_not_hashed() {
        // Regression: `0.08` used to be read as class `08);` etc., corrupting
        // the SCSS so grass failed with `expected "{"` on the *next* line.
        let scss = r"
            $accent-bg: rgba(122, 162, 247, 0.08);
            $code-bg: #131e32;
            .card { background: $accent-bg; box-shadow: 0 2px 12px rgba(0, 0, 0, .06); opacity: 0.5; }
        ";

        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

        assert!(!css.contains("CSS generation failed"), "got: {css}");
        assert!(css.contains("0.08"), "got: {css}");
        assert!(css.contains(".06"), "got: {css}");
        assert!(css.contains("0.5"), "got: {css}");
        assert_eq!(
            class_map.keys().collect::<Vec<_>>(),
            vec![&"card".to_string()]
        );
    }

    #[test]
    fn test_comments_strings_and_interpolation_are_verbatim() {
        let scss = r"
            // .from-line-comment
            /* .from-block-comment */
            $bg: #0b1220;
            :root { --bg: #{$bg}; }
            .card { content: '.from-string'; }
        ";

        let (_css, class_map) = compile_scss_with_hashing(scss, None, false);

        assert!(class_map.contains_key("card"));
        assert!(!class_map.contains_key("from-line-comment"));
        assert!(!class_map.contains_key("from-block-comment"));
        assert!(!class_map.contains_key("from-string"));
        assert!(!class_map.contains_key("bg"), "interpolation left alone");
    }

    #[test]
    fn test_sass_builtin_dot_calls_are_not_hashed() {
        let scss = r"
            @use 'sass:map';
            .card { color: map.get($theme, fg); }
        ";

        let (_css, class_map) = compile_scss_with_hashing(scss, None, false);

        assert!(class_map.contains_key("card"));
        assert!(
            !class_map.contains_key("get"),
            "map.get() is a call, not a class"
        );
    }

    #[test]
    fn test_use_resolves_against_load_paths() {
        let dir = std::env::temp_dir().join(format!("tairitsu-scss-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("_vars.scss"), "$c: #131e32;").unwrap();

        let scss = "@use 'vars' as v; .card { background: v.$c; }";
        let (css, _map) = compile_scss_inner(scss, None, true, std::slice::from_ref(&dir), None);

        std::fs::remove_dir_all(&dir).ok();
        assert!(!css.contains("CSS generation failed"), "got: {css}");
        assert!(css.contains("#131e32"), "got: {css}");
    }

    #[test]
    fn test_file_source_resolves_relative_use_with_extension() {
        // Regression: hikari components write
        // `@use '../../../../theme/styles/variables.scss' as vars;` — an
        // explicit `.scss` extension, which grass's load-path fallback never
        // consults; only from_path gives it a real base directory.
        let dir = std::env::temp_dir().join(format!("tairitsu-scss-use-{}", std::process::id()));
        let theme = dir.join("theme");
        let comp = dir.join("components");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::create_dir_all(&comp).unwrap();
        std::fs::write(theme.join("variables.scss"), "$c: #131e32;").unwrap();
        let main = comp.join("card.scss");
        std::fs::write(
            &main,
            "@use '../theme/variables.scss' as vars; .card { background: vars.$c; }",
        )
        .unwrap();

        let content = std::fs::read_to_string(&main).unwrap();
        let load_paths = vec![comp.clone(), dir.clone()];
        let (css, _map) = compile_scss_inner(&content, None, true, &load_paths, Some(&main));

        std::fs::remove_dir_all(&dir).ok();
        assert!(!css.contains("CSS generation failed"), "got: {css}");
        assert!(css.contains("#131e32"), "got: {css}");
    }

    #[test]
    fn test_parent_selector_and_pseudo_classes_survive() {
        let scss =
            ".foo:hover { color: red; } .bar&.active { color: blue; } :not(.baz) { opacity: 1; }";
        let (css, class_map) = compile_scss_with_hashing(scss, None, false);

        for name in ["foo", "bar", "active", "baz"] {
            let hashed = class_map
                .get(name)
                .unwrap_or_else(|| panic!("{name} hashed"));
            assert!(css.contains(&format!(".{hashed}")), "got: {css}");
        }
        assert!(css.contains(":hover"), "got: {css}");
    }

    #[test]
    fn test_file_source_emits_include_str_for_dep_tracking() {
        // Regression: file-based scss! used to expand without referencing
        // the source file, so cargo never rebuilt on SCSS-only edits and
        // shipped stale CSS. The expansion must route the path through
        // include_str! — rustc then lists it in dep-info.
        let output = wrap_with_dep_tracking(
            std::path::Path::new("C:\\crate\\styles\\layout.scss"),
            quote! { 1 },
        )
        .to_string();
        assert!(output.contains("include_str !"), "got: {output}");
        assert!(
            output.contains("\"C:/crate/styles/layout.scss\""),
            "backslashes normalized to forward slashes, got: {output}"
        );
    }
}
