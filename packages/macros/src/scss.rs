use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use sha2::{Digest, Sha256};

pub struct ScssInput {
    content: String,
    scope: Option<String>,
}

impl syn::parse::Parse for ScssInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit: syn::LitStr = input.parse()?;

        let mut scope = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let ident: syn::Ident = input.parse()?;

            if ident == "scope" {
                input.parse::<syn::Token![:]>()?;
                let scope_lit: syn::LitStr = input.parse()?;
                scope = Some(scope_lit.value());
            }
        }

        Ok(ScssInput {
            content: lit.value(),
            scope,
        })
    }
}

pub fn expand_scss(input: TokenStream) -> TokenStream {
    let scss_input = syn::parse_macro_input!(input as ScssInput);
    let scss_content = scss_input.content;
    let scope = scss_input.scope;

    let (css, class_map) = compile_scss_with_hashing(&scss_content, scope.as_deref());

    let map_entries: Vec<_> = class_map
        .into_iter()
        .map(|(original, hashed)| {
            let original_str = original.as_str();
            let hashed_str = hashed.as_str();
            quote! { (#original_str, #hashed_str) }
        })
        .collect();

    let expanded = quote! {
        {
            let css = #css;
            let class_map = std::collections::HashMap::from([
                #(#map_entries),*
            ]);
            (css, class_map)
        }
    };

    TokenStream::from(expanded)
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
