// props_dsl.rs
// DSL for defining component props with cleaner syntax

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Token, Type, Expr, Ident, Visibility, Attribute, Result, Meta,
};

/// Parsed props field with optional default value from #[default(...)] attribute
pub struct PropsField {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
}

/// Parsed props struct
pub struct PropsInput {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub name: Ident,
    pub fields: Punctuated<PropsField, Token![,]>,
}

impl PropsField {
    /// Extract default value from #[default(...)] attribute
    fn extract_default(attrs: &mut Vec<Attribute>) -> Option<Expr> {
        let mut default_expr = None;
        let mut attrs_to_remove = Vec::new();

        for (i, attr) in attrs.iter().enumerate() {
            if attr.path().is_ident("default") {
                // Parse #[default(expr)]
                if let Meta::List(meta_list) = &attr.meta {
                    let tokens = &meta_list.tokens;
                    if let Ok(expr) = syn::parse2::<Expr>(tokens.clone()) {
                        default_expr = Some(expr);
                        attrs_to_remove.push(i);
                    }
                }
            }
        }

        // Remove default attributes in reverse order to maintain indices
        for i in attrs_to_remove.into_iter().rev() {
            attrs.remove(i);
        }

        default_expr
    }
}

impl Parse for PropsField {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse attributes (if any)
        let mut attrs: Vec<Attribute> = input.call(Attribute::parse_outer)?;

        // Parse visibility (pub or none)
        let vis: Visibility = input.parse()?;

        // Parse field name
        let name: Ident = input.parse()?;

        // Parse colon
        input.parse::<Token![:]>()?;

        // Parse type
        let ty: Type = input.parse()?;

        // Extract default from #[default(...)] attribute
        let default = Self::extract_default(&mut attrs);

        Ok(PropsField {
            attrs,
            vis,
            name,
            ty,
            default,
        })
    }
}

impl Parse for PropsInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse outer attributes (e.g., #[cfg(feature = "tairitsu")])
        let attrs = input.call(Attribute::parse_outer)?;

        // Parse visibility (pub or none)
        let vis: Visibility = input.parse()?;

        // Parse struct keyword
        let _struct_token: Token![struct] = input.parse()?;

        // Parse struct name
        let name: Ident = input.parse()?;

        // Parse brace-enclosed fields
        let content;
        syn::braced!(content in input);

        let fields = content.parse_terminated(PropsField::parse, Token![,])?;

        Ok(PropsInput {
            attrs,
            vis,
            name,
            fields,
        })
    }
}

pub fn expand_define_props(input: PropsInput) -> TokenStream2 {
    let struct_name = &input.name;
    let struct_attrs = &input.attrs;
    let struct_vis = &input.vis;

    // Generate struct fields with #[props(default)] attributes
    let mut struct_fields: Vec<TokenStream2> = Vec::new();
    let mut default_values: Vec<TokenStream2> = Vec::new();
    let mut prop_attrs: Vec<TokenStream2> = Vec::new();

    for field in input.fields.iter() {
        let field_name = &field.name;
        let field_ty = &field.ty;
        let field_vis = &field.vis;

        match &field.default {
            Some(default_expr) => {
                // Field has explicit default value
                prop_attrs.push(quote! {
                    #[props(default = #default_expr)]
                });
                default_values.push(quote! {
                    #field_name: #default_expr
                });
            }
            None => {
                // No default - check if it's Option<T> to auto-default to None
                let ty_str = quote!(#field_ty).to_string();
                if ty_str.contains("Option") {
                    prop_attrs.push(quote! {
                        #[props(default)]
                    });
                    default_values.push(quote! {
                        #field_name: None
                    });
                } else if ty_str.contains("String") {
                    // String defaults to empty
                    prop_attrs.push(quote! {
                        #[props(default)]
                    });
                    default_values.push(quote! {
                        #field_name: String::new()
                    });
                } else if ty_str.contains("Vec") {
                    // Vec defaults to empty
                    prop_attrs.push(quote! {
                        #[props(default)]
                    });
                    default_values.push(quote! {
                        #field_name: Vec::new()
                    });
                } else {
                    // Try to use Default trait
                    prop_attrs.push(quote! {
                        #[props(default)]
                    });
                    default_values.push(quote! {
                        #field_name: Default::default()
                    });
                }
            }
        }

        struct_fields.push(quote! {
            #field_vis #field_name: #field_ty
        });
    }

    // Generate the struct with Props derive
    // Use ::tairitsu_macros::Props to ensure the derive macro is found
    let expanded = quote! {
        #(#struct_attrs)*
        #[derive(Clone, PartialEq, ::tairitsu_macros::Props)]
        #struct_vis struct #struct_name {
            #(
                #prop_attrs
                #struct_fields,
            )*
        }

        impl Default for #struct_name {
            fn default() -> Self {
                Self {
                    #(#default_values,)*
                }
            }
        }
    };

    expanded
}
