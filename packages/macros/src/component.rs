use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, FnArg, Ident, ItemFn, Pat, PatType, Result};

pub fn expand_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = match syn::parse(item.clone()) {
        Ok(f) => f,
        Err(e) => {
            let msg = format!(
                "#[component] can only be applied to functions. \
                 Consider adding `fn` before the function name.\n{}",
                e,
            );
            return syn::Error::new(e.span(), msg).to_compile_error().into();
        }
    };

    match expand_component_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_component_impl(mut input: ItemFn) -> Result<TokenStream2> {
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_return = &input.sig.output;

    // Check if the function takes a single Props parameter (e.g., `props: BackgroundProps`)
    // In this case, we don't generate the Props struct
    let mut uses_existing_props = false;
    let mut existing_props_name: Option<syn::Type> = None;

    if input.sig.inputs.len() == 1 {
        if let Some(FnArg::Typed(pat_type)) = input.sig.inputs.first() {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                // Check if param name is "props" and type ends with "Props"
                if pat_ident.ident == "props" {
                    if let syn::Type::Path(type_path) = &*pat_type.ty {
                        if let Some(segment) = type_path.path.segments.last() {
                            if segment.ident.to_string().ends_with("Props") {
                                uses_existing_props = true;
                                existing_props_name = Some((*pat_type.ty).clone());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut fields = Vec::new();
    let mut field_defaults = Vec::new();
    let mut builder_methods = Vec::new();
    let mut prop_names = Vec::new();
    let mut prop_has_defaults = Vec::new(); // Track which props have defaults

    // Strip doc comments from function parameters and extract info
    for arg in &mut input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Remove doc comments from parameter attributes
            pat_type.attrs.retain(|attr| !attr.path().is_ident("doc"));

            // Skip extraction if using existing props
            if !uses_existing_props {
                let (name, has_default, _is_children) = extract_arg_info(pat_type)?;
                let ty = (*pat_type.ty).clone();

                prop_names.push(name.clone());
                prop_has_defaults.push(has_default);

                // For non-default fields, wrap in Option<T>
                let field_ty = if has_default {
                    quote! { #ty }
                } else {
                    quote! { Option<#ty> }
                };

                fields.push(quote! {
                    pub #name: #field_ty
                });

                if has_default {
                    field_defaults.push(quote! {
                        #name: Default::default()
                    });
                } else {
                    field_defaults.push(quote! {
                        #name: None
                    });
                }

                // For builder methods, unwrap Option for required fields
                let builder_body = if has_default {
                    quote! { self.#name = #name; }
                } else {
                    quote! { self.#name = Some(#name); }
                };

                builder_methods.push(quote! {
                    pub fn #name(mut self, #name: #ty) -> Self {
                        #builder_body
                        self
                    }
                });
            }
        }
    }

    let props_name = format_ident!("{}Props", to_pascal_case(&fn_name.to_string()));
    let builder_name = format_ident!("{}Builder", to_pascal_case(&fn_name.to_string()));

    let props_struct = if uses_existing_props {
        // Don't generate Props struct - use existing one
        quote! {}
    } else {
        quote! {
            #[derive(Debug, Clone)]
            #fn_vis struct #props_name {
                #(#fields),*
            }
        }
    };

    let builder_struct = if uses_existing_props {
        // Don't generate builder for existing props
        quote! {}
    } else {
        quote! {
            #[derive(Debug, Clone)]
            #fn_vis struct #builder_name{
                #(#fields),*
            }
        }
    };

    let props_impl = if uses_existing_props {
        quote! {}
    } else {
        quote! {
            impl #props_name {
                pub fn builder() -> #builder_name {
                    #builder_name {
                        #(#field_defaults),*
                    }
                }
            }

            impl Default for #props_name {
                fn default() -> Self {
                    Self {
                        #(#field_defaults),*
                    }
                }
            }
        }
    };

    let builder_impl = if uses_existing_props {
        quote! {}
    } else {
        quote! {
            impl #builder_name{
                #(#builder_methods)*

                pub fn build(self) -> #props_name {
                    #props_name {
                        #(#prop_names: self.#prop_names),*
                    }
                }
            }

            impl Default for #builder_name{
                fn default() -> Self {
                    Self {
                        #(#field_defaults),*
                    }
                }
            }
        }
    };

    // Build cleaned parameters without doc comments
    let mut cleaned_inputs: Vec<FnArg> = Vec::new();
    for arg in &input.sig.inputs {
        if let FnArg::Typed(mut pat_type) = arg.clone() {
            // Remove doc comments from parameter attributes
            pat_type.attrs.retain(|attr| !attr.path().is_ident("doc"));
            cleaned_inputs.push(FnArg::Typed(pat_type));
        } else {
            cleaned_inputs.push(arg.clone());
        }
    }

    // Create the function
    let original_fn = if uses_existing_props {
        let props_type = existing_props_name
            .expect("existing_props_name must be set when uses_existing_props is true");
        quote! {
            #[allow(non_snake_case)]
            #[allow(unused_braces)]
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            #[allow(clippy::needless_update)]
            #fn_vis fn #fn_name(props: #props_type) #fn_return {
                #fn_block
            }
        }
    } else {
        let prop_bindings: Vec<_> = prop_names
            .iter()
            .zip(prop_has_defaults.iter())
            .map(|(name, has_default)| {
                if *has_default {
                    quote! { let #name = props.#name; }
                } else {
                    let err_msg = format!("`{}` is required but was not provided", name);
                    quote! { let #name = props.#name.expect(#err_msg); }
                }
            })
            .collect();

        quote! {
            #[allow(non_snake_case)]
            #[allow(unused_braces)]
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            #[allow(clippy::needless_update)]
            #fn_vis fn #fn_name(props: #props_name) #fn_return {
                #(#prop_bindings)*
                #fn_block
            }
        }
    };

    let expanded = quote! {
        #props_struct
        #builder_struct
        #props_impl
        #builder_impl
        #original_fn
    };

    Ok(expanded)
}

fn extract_arg_info(pat_type: &PatType) -> Result<(Ident, bool, bool)> {
    let name = if let Pat::Ident(pat_ident) = &*pat_type.pat {
        pat_ident.ident.clone()
    } else {
        return Err(syn::Error::new_spanned(
            pat_type,
            "Expected identifier pattern",
        ));
    };

    let has_default = has_props_attribute(&pat_type.attrs, "default");
    let is_children = has_props_attribute(&pat_type.attrs, "children");

    Ok((name, has_default, is_children))
}

fn has_props_attribute(attrs: &[Attribute], inner_name: &str) -> bool {
    attrs.iter().any(|attr| {
        // Check for #[props(...)] pattern
        if attr.path().is_ident("props") {
            if let syn::Meta::List(meta_list) = &attr.meta {
                // Parse nested meta items properly to avoid substring false positives.
                // e.g. #[props(default_value)] should NOT match inner_name = "default".
                // We use syn::parse2 with a custom parser for a comma-separated list of Meta.
                return syn::parse2::<PropsMetaList>(meta_list.tokens.clone())
                    .ok()
                    .is_some_and(|list| list.0.iter().any(|m| m.path().is_ident(inner_name)));
            }
        }
        // Also check for direct #[default] for backward compatibility
        if attr.path().is_ident(inner_name) {
            return true;
        }
        false
    })
}

/// Helper to parse a comma-separated list of `syn::Meta` items.
struct PropsMetaList(Vec<syn::Meta>);

impl syn::parse::Parse for PropsMetaList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut metas = Vec::new();
        while !input.is_empty() {
            let meta = input.parse::<syn::Meta>()?;
            metas.push(meta);
            if input.peek(syn::Token![,]) {
                let _ = input.parse::<syn::Token![,]>();
            }
        }
        Ok(PropsMetaList(metas))
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(
                ch.to_uppercase()
                    .next()
                    .expect("to_uppercase always yields at least one char"),
            );
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_has_props_attribute_detects_default() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[props(default)])];
        assert!(has_props_attribute(&attrs, "default"));
    }

    #[test]
    fn test_has_props_attribute_detects_children() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[props(children)])];
        assert!(has_props_attribute(&attrs, "children"));
    }

    #[test]
    fn test_has_props_attribute_no_false_positive() {
        // #[props(default_value = "...")] should NOT match inner_name = "default"
        let attrs: Vec<Attribute> = vec![parse_quote!(#[props(default_value = "foo")])];
        assert!(
            !has_props_attribute(&attrs, "default"),
            "default_value should not match 'default'"
        );
    }

    #[test]
    fn test_has_props_attribute_multiple() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[props(default, children)])];
        assert!(has_props_attribute(&attrs, "default"));
        assert!(has_props_attribute(&attrs, "children"));
    }

    #[test]
    fn test_has_props_attribute_no_match() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[props(required)])];
        assert!(!has_props_attribute(&attrs, "default"));
        assert!(!has_props_attribute(&attrs, "children"));
    }

    #[test]
    fn test_has_props_attribute_direct_default() {
        let attrs: Vec<Attribute> = vec![parse_quote!(#[default])];
        assert!(has_props_attribute(&attrs, "default"));
    }
}
