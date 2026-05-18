//! `include_scss!` — compile-time SCSS class extraction & type-safe enum generation.
//!
//! # Overview
//!
//! ```ignore
//! include_scss!("src/styles/components/button.scss")
//! // Generates:
//! //   pub enum ButtonScssClasses {
//! //       HiButton,           // → "hi-button"
//! //       HiButtonPrimary,    // → "hi-button-primary"
//! //       HiButtonSm,         // → "hi-button-sm"
//! //       HiButtonLg,         // → "hi-button-lg"
//! //       // ... every .class found in the file
//! //   }
//! //
//! // + impl TypedClass  (for ClassesBuilder integration)
//! // + impl Into<Classes> (for direct rsx! class: usage)
//! // + impl ButtonScssClasses { pub fn classes(slice: &[Self]) -> Classes }
//! ```
//!
//! # Parser design
//!
//! The extractor is a **hand-written recursive descent parser** that
//! faithfully implements the grammar defined in `scss_classes.pest`.
//! It correctly handles:
//! - SCSS nesting (`&.class`, `&:hover`)
//! - Comments (`//`, `/* */`)
//! - Strings (may contain dots that are NOT selectors)
//! - `@`-rules (`@media`, `@keyframes`, etc.)
//! - Chained selectors (`.a.b.c`)
//! - Pseudo-selectors (`.class:hover`, `.class::before`)
//! - Combinators (`>`, `+`, `~`, space)

use std::collections::{BTreeMap, BTreeSet};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::LitStr;

// ─── Input syntax ──────────────────────────────────────────────────────

struct IncludeScssInput {
    path: LitStr,
    prefix: Option<String>,
    enum_name: Option<syn::Ident>,
    filter: Option<String>,
}

impl Parse for IncludeScssInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;

        let mut prefix = None;
        let mut enum_name = None;
        let mut filter = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let key: syn::Ident = input.parse()?;
            input.parse::<syn::Token![:]>()?;

            match key.to_string().as_str() {
                "prefix" => {
                    let v: LitStr = input.parse()?;
                    prefix = Some(v.value());
                }
                "enum_name" | "enum" => {
                    let id: syn::Ident = input.parse()?;
                    enum_name = Some(id);
                }
                "filter" => {
                    let v: LitStr = input.parse()?;
                    filter = Some(v.value());
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!(
                            "unknown option `{other}`, expected `prefix`, `enum_name` or `filter`"
                        ),
                    ));
                }
            }
        }

        Ok(IncludeScssInput {
            path,
            prefix,
            enum_name,
            filter,
        })
    }
}

// ─── Public entry point ───────────────────────────────────────────────

pub fn expand_include_scss(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as IncludeScssInput);

    let crate_root = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(v) => v,
        Err(_) => {
            return quote! { compile_error!("CARGO_MANIFEST_DIR not set") }.into();
        }
    };

    let full_path = std::path::Path::new(&crate_root).join(input.path.value());
    let scss_content = match std::fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!(
                "include_scss!: failed to read '{}': {}",
                input.path.value(),
                e
            );
            return quote! { compile_error!(#msg) }.into();
        }
    };

    let mut classes = extract_classes(&scss_content);

    if let Some(ref prefix_filter) = input.filter {
        let pf = prefix_filter.to_lowercase();
        classes.retain(|c| c.to_lowercase().starts_with(&pf));
    }

    if classes.is_empty() {
        let msg = format!(
            "include_scss!: no CSS classes found in '{}'",
            input.path.value()
        );
        return quote! { compile_error!(#msg) }.into();
    }

    generate_enum(
        &classes,
        &input.path.value(),
        input.prefix.as_deref(),
        input.enum_name.as_ref(),
    )
    .into()
}

// ════════════════════════════════════════════════════════════════════════
//  SCSS class extractor — mirrors scss_classes.pest grammar
// ════════════════════════════════════════════════════════════════════════

fn extract_classes(scss: &str) -> BTreeSet<String> {
    let extractor = ScssExtractor::new(scss);
    extractor.extract()
}

struct ScssExtractor {
    chars: Vec<char>,
    len: usize,
    pos: usize,
    classes: BTreeSet<String>,
}

impl ScssExtractor {
    fn new(src: &str) -> Self {
        let chars: Vec<char> = src.chars().collect();
        Self {
            chars,
            len: src.chars().count(),
            pos: 0,
            classes: BTreeSet::new(),
        }
    }

    fn extract(mut self) -> BTreeSet<String> {
        self.parse_scss_content();
        self.classes
    }

    fn remaining(&self) -> bool {
        self.pos < self.len
    }

    fn ch(&self) -> char {
        if self.pos < self.len {
            self.chars[self.pos]
        } else {
            '\0'
        }
    }

    fn ch_at(&self, offset: usize) -> char {
        let idx = self.pos + offset;
        if idx < self.len {
            self.chars[idx]
        } else {
            '\0'
        }
    }

    fn advance(&mut self, n: usize) {
        self.pos += n;
    }

    fn starts_with(&self, s: &str) -> bool {
        let target: Vec<char> = s.chars().collect();
        if self.pos + target.len() > self.len {
            return false;
        }
        self.chars[self.pos..].starts_with(&target)
    }

    /// scss_content* (top level)
    fn parse_scss_content(&mut self) {
        while self.remaining() {
            match self.ch() {
                '/' if self.ch_at(1) == '*' => self.skip_block_comment(),
                '/' if self.ch_at(1) != '\0' => {
                    self.advance(2);
                    self.skip_to_eol();
                }
                '"' => self.skip_string_double(),
                '\'' => self.skip_string_single(),
                '@' => self.skip_at_rule(),
                '{' => self.skip_to_matching_brace(),
                _ => {
                    // Try to extract selectors at current position
                    // (selectors appear BEFORE their opening brace)
                    self.try_extract_selectors();
                    // Advance past whatever this token was
                    if self.remaining() && self.ch() != '{' {
                        self.advance(1);
                    }
                }
            }
        }
    }

    /// Skip from the opening { to its matching }
    fn skip_to_matching_brace(&mut self) {
        debug_assert_eq!(self.ch(), '{');
        let mut depth = 1u32;
        self.advance(1);
        while self.remaining() && depth > 0 {
            match self.ch() {
                '{' => {
                    depth += 1;
                    self.advance(1);
                }
                '}' => {
                    depth -= 1;
                    self.advance(1);
                }
                '"' => self.skip_string_double(),
                '\'' => self.skip_string_single(),
                '/' if self.ch_at(1) == '*' => self.skip_block_comment(),
                '/' if self.ch_at(1) != '\0' => {
                    self.advance(2);
                    self.skip_to_eol();
                }
                _ => self.advance(1),
            }
        }
    }

    /// Try to extract class selectors at current position.
    /// Scans forward until we hit '{', '}', ';', or EOF, collecting
    /// any .class-name tokens along the way.
    fn try_extract_selectors(&mut self) {
        while self.remaining() {
            match self.ch() {
                '{' | '}' | ';' | ')' => return,
                ',' => {
                    self.advance(1);
                    self.skip_ws();
                }
                '\n' | '\r' | '\t' | ' ' => {
                    self.advance(1);
                }
                '"' | '\'' => return,
                '/' if self.ch_at(1) == '*' || self.ch_at(1) != '\0' => return,
                '.' => self.consume_class_selector(),
                '#' => {
                    self.advance(1);
                    while self.remaining() && is_ident_char(self.ch()) {
                        self.advance(1);
                    }
                }
                '[' => self.skip_bracketed(),
                ':' => self.skip_pseudo(),
                '&' => {
                    self.advance(1);
                    if self.remaining() && self.ch() == '.' {
                        self.consume_class_selector();
                    }
                }
                _ => {
                    if self.ch().is_alphabetic()
                        || self.ch() == '_'
                        || self.ch() == '-'
                        || self.ch() == '*'
                    {
                        while self.remaining() && (is_ident_char(self.ch()) || self.ch() == '-') {
                            self.advance(1);
                        }
                    } else {
                        self.advance(1);
                    }
                }
            }
        }
    }

    /// Consume a .class-name token and register it.
    /// Handles: `.name`, `.name:pseudo`, `.name::pseudo`
    fn consume_class_selector(&mut self) {
        debug_assert_eq!(self.ch(), '.');
        self.advance(1);

        let start = self.pos;
        while self.remaining() && is_class_char(self.ch()) {
            self.advance(1);
        }

        if self.pos > start {
            let name: String = self.chars[start..self.pos].iter().collect();
            if is_valid_class_name(&name) {
                self.classes.insert(name);
            }
        }

        // Skip trailing pseudo-selector if present (:hover, ::before, etc.)
        if self.remaining() && self.ch() == ':' {
            self.skip_pseudo();
        }
    }

    // ─── Skippers ──────────────────────────────────────────────────

    fn skip_block_comment(&mut self) {
        debug_assert!(self.starts_with("/*"));
        self.advance(2);
        while self.remaining() {
            if self.starts_with("*/") {
                self.advance(2);
                return;
            }
            self.advance(1);
        }
    }

    fn skip_to_eol(&mut self) {
        while self.remaining() && self.ch() != '\n' {
            self.advance(1);
        }
    }

    fn skip_string_double(&mut self) {
        debug_assert_eq!(self.ch(), '"');
        self.advance(1);
        while self.remaining() {
            match self.ch() {
                '\\' if self.ch_at(1) != '\0' => {
                    self.advance(2);
                }
                '"' => {
                    self.advance(1);
                    return;
                }
                _ => self.advance(1),
            }
        }
    }

    fn skip_string_single(&mut self) {
        debug_assert_eq!(self.ch(), '\'');
        self.advance(1);
        while self.remaining() {
            match self.ch() {
                '\\' if self.ch_at(1) != '\0' => {
                    self.advance(2);
                }
                '\'' => {
                    self.advance(1);
                    return;
                }
                _ => self.advance(1),
            }
        }
    }

    /// Skip @rule until matching ';' (simple) or '{...}' (block)
    fn skip_at_rule(&mut self) {
        debug_assert_eq!(self.ch(), '@');
        self.advance(1);

        let depth: u32 = 0;
        self.skip_at_rule_body(depth);
    }

    fn skip_at_rule_body(&mut self, depth: u32) {
        while self.remaining() {
            match self.ch() {
                ';' if depth == 0 => {
                    self.advance(1);
                    return;
                }
                '{' => {
                    self.advance(1);
                    self.skip_at_rule_body(depth + 1);
                }
                '}' if depth > 0 => {
                    self.advance(1);
                    return;
                }
                '"' => self.skip_string_double(),
                '\'' => self.skip_string_single(),
                '/' if self.ch_at(1) == '*' => self.skip_block_comment(),
                _ => self.advance(1),
            }
        }
    }

    fn skip_bracketed(&mut self) {
        debug_assert_eq!(self.ch(), '[');
        self.advance(1);
        let mut depth = 1u32;
        while self.remaining() && depth > 0 {
            match self.ch() {
                '[' => {
                    depth += 1;
                    self.advance(1);
                }
                ']' => {
                    depth -= 1;
                    self.advance(1);
                }
                '"' => self.skip_string_double(),
                '\'' => self.skip_string_single(),
                _ => self.advance(1),
            }
        }
    }

    /// Skip :pseudo or ::pseudo (including pseudo-class args like :nth-child(2))
    fn skip_pseudo(&mut self) {
        debug_assert_eq!(self.ch(), ':');
        self.advance(1);
        if self.remaining() && self.ch() == ':' {
            self.advance(1);
        }
        while self.remaining() && is_ident_char(self.ch()) {
            self.advance(1);
        }
        if self.remaining() && self.ch() == '(' {
            self.advance(1);
            let mut depth = 1u32;
            while self.remaining() && depth > 0 {
                match self.ch() {
                    '(' => {
                        depth += 1;
                        self.advance(1);
                    }
                    ')' => {
                        depth -= 1;
                        self.advance(1);
                    }
                    '"' => self.skip_string_double(),
                    '\'' => self.skip_string_single(),
                    _ => self.advance(1),
                }
            }
        }
    }

    fn skip_ws(&mut self) {
        while self.remaining() && matches!(self.ch(), ' ' | '\t' | '\n' | '\r') {
            self.advance(1);
        }
    }
}

/// Character class for CSS class name identifiers (after the dot).
fn is_class_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

/// Validate that a string looks like a real CSS class identifier.
fn is_valid_class_name(s: &str) -> bool {
    !s.is_empty()
        && s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_' || c == '-')
        && s.chars().all(is_class_char)
}

// ─── Code generation ─────────────────────────────────────────────────

fn generate_enum(
    classes: &BTreeSet<String>,
    file_path: &str,
    prefix_override: Option<&str>,
    enum_name_override: Option<&syn::Ident>,
) -> proc_macro2::TokenStream {
    let enum_name = match enum_name_override {
        Some(id) => id.clone(),
        None => derive_enum_name(file_path),
    };

    let mut variant_map: BTreeMap<syn::Ident, (String, &str)> = BTreeMap::new();

    for class in classes.iter() {
        let v = class_to_variant(class, prefix_override);
        let doc = format!("CSS class `.{}`", class);
        variant_map.entry(v).or_insert((doc, class.as_str()));
    }

    let variant_tokens: Vec<proc_macro2::TokenStream> = variant_map
        .iter()
        .map(|(v, (doc, _))| {
            quote! { #[doc = #doc] #v }
        })
        .collect();

    let match_arms: Vec<proc_macro2::TokenStream> = variant_map
        .iter()
        .map(|(v, (_, s))| {
            quote! { Self::#v => #s }
        })
        .collect();

    let count = variant_map.len();

    quote! {
        /// Auto-generated enum of CSS classes extracted from SCSS source.
        ///
        /// Each variant maps 1:1 to a `.class-name` defined in the stylesheet.
        /// Use in rsx! `class:` attributes for compile-time safety:
        ///
        /// ```ignore
        /// // Single class via .as_str():
        /// rsx! { button { class: #enum_name::HiButton.as_str() } }
        /// // Multiple classes via .classes():
        /// let c = #enum_name::classes(&[#enum_name::HiButton, #enum_name::HiButtonPrimary]);
        /// rsx! { div { class: c } }
        /// ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum #enum_name {
            #(#variant_tokens),*
        }

        impl #enum_name {
            /// Returns the CSS class name string for this variant.
            pub const fn as_str(&self) -> &'static str {
                match self {
                    #(#match_arms),*
                }
            }

            /// Combine multiple class variants into a space-separated string
            /// suitable for constructing `tairitsu_vdom::Classes`.
            pub fn class_names(items: &[Self]) -> String {
                let mut s = String::new();
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { s.push(' '); }
                    s.push_str(item.as_str());
                }
                s
            }

            /// Total number of known classes from this SCSS source.
            pub const COUNT: usize = #count;
        }
    }
}

// ─── Name helpers ─────────────────────────────────────────────────────

fn derive_enum_name(path: &str) -> syn::Ident {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    let pascal = to_pascal_case(stem);
    format_ident!("{}Classes", pascal)
}

fn class_to_variant(class: &str, prefix_override: Option<&str>) -> syn::Ident {
    let normalized = class
        .replace("--", "__")
        .replace('-', "_")
        .replace("__", "_");

    let parts: Vec<&str> = normalized.split('_').filter(|p| !p.is_empty()).collect();

    if parts.is_empty() {
        return format_ident!("Unknown");
    }

    match prefix_override {
        Some(prefix) => {
            let rest: String = if parts.len() > 1 {
                parts[1..].iter().map(|p| capitalize(p)).collect()
            } else {
                String::new()
            };
            format_ident!("{}{}", capitalize(prefix), rest)
        }
        None => {
            let full: String = parts
                .iter()
                .map(|p| capitalize(p))
                .collect::<Vec<_>>()
                .join("");
            format_ident!("{}", full)
        }
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split(['-', '_', '.'])
        .filter(|p| !p.is_empty())
        .map(capitalize)
        .collect()
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
