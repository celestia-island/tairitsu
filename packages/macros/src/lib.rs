use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Derives WitCommand trait for an enum, automatically generating Response type and command routing
///
/// # Example
/// ```ignore
/// #[derive(WitCommand)]
/// #[wit_response(FileSystemResponse)]
/// enum FileSystemCommands {
///     Read { path: String },
///     Write { path: String, data: Vec<u8> },
/// }
/// ```
#[proc_macro_derive(WitCommand, attributes(wit_response))]
pub fn derive_wit_command(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract response type from attribute or default to String
    let response_type = extract_response_type(&input.attrs);

    // Generate command name arms from enum variants
    let command_name_arms = if let Data::Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|variant| {
                let variant_name = &variant.ident;
                let cmd_name_str = to_kebab_case(&variant_name.to_string());
                quote! {
                    #name::#variant_name { .. } => #cmd_name_str
                }
            })
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    let expanded = quote! {
        impl tairitsu::wit_registry::WitCommand for #name {
            type Response = #response_type;

            fn command_name(&self) -> &'static str {
                match self {
                    #(#command_name_arms),*
                }
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_response_type(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("wit_response") {
            if let Ok(ty) = attr.parse_args::<syn::Type>() {
                return quote! { #ty };
            }
        }
    }
    quote! { String }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

/// Generates WIT command enums and handlers from WIT interface definitions
///
/// # Example
/// ```ignore
/// wit_interface! {
///     interface filesystem {
///         read: func(path: String) -> Result<Vec<u8>, String>;
///         write: func(path: String, data: Vec<u8>) -> Result<(), String>;
///     }
/// }
/// ```
#[proc_macro]
pub fn wit_interface(input: TokenStream) -> TokenStream {
    // Parse the WIT-like syntax
    let ast = parse_macro_input!(input as WitInterface);

    let interface_name = &ast.name;
    let commands_enum_name = syn::Ident::new(
        &format!("{}Commands", capitalize(&interface_name.to_string())),
        interface_name.span(),
    );
    let response_enum_name = syn::Ident::new(
        &format!("{}Response", capitalize(&interface_name.to_string())),
        interface_name.span(),
    );

    let mut command_variants = Vec::new();
    let mut response_variants = Vec::new();
    let mut command_name_arms = Vec::new();

    for func in &ast.functions {
        let variant_name = syn::Ident::new(&capitalize(&func.name.to_string()), func.name.span());

        // Build command variant
        let params: Vec<_> = func
            .params
            .iter()
            .map(|(name, ty)| {
                let field_name = syn::Ident::new(&name.to_string(), name.span());
                quote! { #field_name: #ty }
            })
            .collect();

        command_variants.push(quote! {
            #variant_name { #(#params),* }
        });

        // Build response variant
        if let Some(ret_ty) = &func.return_type {
            response_variants.push(quote! {
                #variant_name(#ret_ty)
            });
        }

        // Build command name mapping
        let cmd_name_str = func.name.to_string();
        command_name_arms.push(quote! {
            #commands_enum_name::#variant_name { .. } => #cmd_name_str
        });
    }

    let expanded = quote! {
        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types)]
        pub enum #commands_enum_name {
            #(#command_variants),*
        }

        #[derive(Debug, Clone)]
        #[allow(non_camel_case_types)]
        pub enum #response_enum_name {
            #(#response_variants),*
        }

        impl tairitsu::wit_registry::WitCommand for #commands_enum_name {
            type Response = #response_enum_name;

            fn command_name(&self) -> &'static str {
                match self {
                    #(#command_name_arms),*
                }
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(expanded)
}

// AST structures for parsing WIT-like syntax
struct WitInterface {
    name: syn::Ident,
    functions: Vec<WitFunction>,
}

struct WitFunction {
    name: syn::Ident,
    params: Vec<(syn::Ident, syn::Type)>,
    return_type: Option<syn::Type>,
}

impl syn::parse::Parse for WitInterface {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Parse "interface" keyword
        let interface_keyword: syn::Ident = input.parse()?;
        if interface_keyword != "interface" {
            return Err(syn::Error::new(
                interface_keyword.span(),
                "expected 'interface' keyword",
            ));
        }
        let name: syn::Ident = input.parse()?;

        let content;
        syn::braced!(content in input);

        let mut functions = Vec::new();
        while !content.is_empty() {
            functions.push(content.parse()?);
        }

        Ok(WitInterface { name, functions })
    }
}

impl syn::parse::Parse for WitFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        input.parse::<syn::Ident>()?; // "func"

        let content;
        syn::parenthesized!(content in input);

        let mut params = Vec::new();
        while !content.is_empty() {
            let param_name: syn::Ident = content.parse()?;
            content.parse::<syn::Token![:]>()?;
            let param_type: syn::Type = content.parse()?;
            params.push((param_name, param_type));

            if !content.is_empty() {
                content.parse::<syn::Token![,]>()?;
            }
        }

        let return_type = if input.peek(syn::Token![->]) {
            input.parse::<syn::Token![->]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        input.parse::<syn::Token![;]>()?;

        Ok(WitFunction {
            name,
            params,
            return_type,
        })
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}
