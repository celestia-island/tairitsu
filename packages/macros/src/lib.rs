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

/// Attribute macro to export a function via WIT for WASM guest modules
///
/// This macro wraps a function and generates the necessary WIT bindings
/// for it to be callable from the host.
///
/// # Example
/// ```ignore
/// #[export_wit]
/// pub fn my_function(input: String) -> Result<String, String> {
///     Ok(format!("Processed: {}", input))
/// }
/// ```
#[proc_macro_attribute]
pub fn export_wit(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as syn::ItemFn);

    // Generate the WIT export wrapper
    let expanded = quote! {
        // Original function (non-WASM target)
        #[cfg(not(target_family = "wasm"))]
        #input_fn

        // WASM export wrapper
        #[cfg(target_family = "wasm")]
        #input_fn
    };

    TokenStream::from(expanded)
}

/// Macro to generate complete WIT guest implementation
///
/// This macro generates the necessary code for a WASM guest module
/// using the Tairitsu WIT interfaces.
///
/// # Example
/// ```ignore
/// wit_guest_impl! {
///     name: "my-guest",
///     version: "0.1.0",
///     features: ["feature1", "feature2"],
///
///     exports: {
///         init: || Ok(()),
///         process: |input| Ok(format!("Hello, {}!", input)),
///     },
/// }
/// ```
#[proc_macro]
pub fn wit_guest_impl(input: TokenStream) -> TokenStream {
    let _ast = parse_macro_input!(input as WitGuestImpl);

    // Generate the guest implementation
    let expanded = quote! {
        // Guest module for non-WASM targets (native/testing)
        #[cfg(not(target_family = "wasm"))]
        pub mod guest {
            use super::*;

            pub fn init() -> Result<(), String> {
                Ok(())
            }

            pub fn process(input: String) -> Result<String, String> {
                Ok(format!("Processed: {}", input))
            }

            pub fn get_info() -> tairitsu::wit_helper::GuestInfo {
                tairitsu::wit_helper::GuestInfo {
                    name: "tairitsu-guest".to_string(),
                    version: "0.1.0".to_string(),
                    features: vec!["wit-native".to_string()],
                }
            }
        }

        // For WASM targets, these will be implemented via WIT bindings
        #[cfg(target_family = "wasm")]
        pub mod guest {
            use super::*;

            // WIT bindings will be generated by wit-bindgen
            wit_bindgen::generate!({
                path: "../../wit",
                world: "tairitsu",
                exports: {
                    "tairitsu:core/guest-api": MyGuest
                }
            });

            // Implement the guest API trait
            struct MyGuest;

            impl exports::tairitsu::core::guest_api::Guest for MyGuest {
                fn init() -> Result<(), String> {
                    Ok(())
                }

                fn process(input: String) -> Result<String, String> {
                    Ok(format!("Processed from WASM: {}", input))
                }

                fn get_info() -> exports::tairitsu::core::guest_api::Info {
                    exports::tairitsu::core::guest_api::Info {
                        name: "tairitsu-wasm-guest".to_string(),
                        version: "0.1.0".to_string(),
                        features: vec!["wit-native".to_string(), "wasm".to_string()],
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

// AST structure for wit_guest_impl macro
struct WitGuestImpl {
    // Placeholder for parsing the macro input
}

impl syn::parse::Parse for WitGuestImpl {
    fn parse(_input: syn::parse::ParseStream) -> syn::Result<Self> {
        // For now, just succeed without parsing
        // TODO: Implement proper parsing of the macro syntax
        Ok(WitGuestImpl {})
    }
}

/// Helper macro to simplify wasmtime component bindgen usage
///
/// This macro wraps wasmtime::component::bindgen! with a simpler interface.
/// Note: This is a procedural macro placeholder. For actual bindgen functionality,
/// use wasmtime::component::bindgen! directly.
///
/// # Example
/// ```ignore
/// use tairitsu_macros::wit_world;
///
/// // This will generate the bindings for the specified world
/// wit_world!("my-package:my-world", "./wit");
/// ```
#[proc_macro]
pub fn wit_world(input: TokenStream) -> TokenStream {
    // Parse input: "package:world", "./wit/path"
    let input_str = input.to_string();

    // Remove quotes if present
    let input_str = input_str.trim_matches('"').trim_matches('\'');

    // Split by comma to get world and path
    let parts: Vec<&str> = input_str.split(',').collect();
    let world = parts.first().map(|s| s.trim()).unwrap_or("");
    let wit_path = parts.get(1).map(|s| s.trim()).unwrap_or("./wit");

    // Generate code that uses wasmtime::component::bindgen!
    let _world_ident = syn::Ident::new(
        &world.replace([':', '-'], "_"),
        proc_macro2::Span::call_site(),
    );

    let expanded = quote! {
        // This is a placeholder. In a real implementation, this would
        // invoke wasmtime::component::bindgen! with the appropriate parameters.
        //
        // For now, users should use wasmtime::component::bindgen! directly:
        //
        // wasmtime::component::bindgen!({
        //     path: #wit_path,
        //     world: #world,
        // });
        //
        // Or generate the bindings using wit-bindgen-cli:
        // wit-bindgen rust --out-dir bindings #wit_path

        compile_error!(concat!(
            "wit_world! macro is a placeholder. Use wasmtime::component::bindgen! directly:\n",
            "wasmtime::component::bindgen!({\n",
            "    path: \"",
            #wit_path,
            "\",\n",
            "    world: \"",
            #world,
            "\",\n",
            "});"
        ));
    };

    TokenStream::from(expanded)
}

/// Helper macro to automatically generate add_to_linker calls
///
/// This macro simplifies the process of registering host functions with the linker.
///
/// # Example
/// ```ignore
/// use tairitsu_macros::register_host;
///
/// struct MyHost {
///     // your host state
/// }
///
/// register_host! {
///     MyHost,
///     functions: {
///         my_function: |state, arg1, arg2| {
///             // implementation
///             Ok(())
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn register_host(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as syn::ItemStruct);

    // Parse the struct to extract host type and functions
    // For now, this is a placeholder that shows the intended usage

    let expanded = quote! {
        // This is a placeholder implementation.
        //
        // A full implementation would:
        // 1. Parse the host struct and its methods
        // 2. Generate trait implementations for WIT interfaces
        // 3. Generate add_to_linker boilerplate
        //
        // For now, users should manually implement the WIT traits
        // and call add_to_linker themselves:
        //
        // impl MyWit for MyHost {
        //     fn my_function(&mut self, arg1: String, arg2: u32) -> Result<(), String> {
        //         // implementation
        //     }
        // }
        //
        // Then use:
        // MyWit::add_to_linker(&mut linker, |state| &mut state.my_data)?;

        compile_error!(
            "register_host! macro is a placeholder. \
             Manually implement WIT traits and use add_to_linker for now."
        );
    };

    TokenStream::from(expanded)
}

/// Derive macro to automatically implement Tool for a struct
///
/// This derive macro implements the Tool trait for structs that have
/// an invoke_json method.
///
/// # Example
/// ```ignore
/// use tairitsu_macros::Tool;
/// use tairitsu::json::Tool;
///
/// #[derive(Tool)]
/// struct MyTool {
///     // fields
/// }
///
/// impl MyTool {
///     fn invoke_json(&self, json: &str) -> Result<String> {
///         // implementation
///     }
/// }
/// ```
#[proc_macro_derive(AsTool, attributes(tool_name))]
pub fn derive_as_tool(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract tool name from attribute or use struct name
    let tool_name = extract_tool_name(&input.attrs, &name.to_string());

    let expanded = quote! {
        impl tairitsu::json::Tool for #name {
            fn invoke_json(&self, json: &str) -> anyhow::Result<String> {
                // Delegate to the struct's invoke_json method
                self.invoke_json(json)
            }

            fn name(&self) -> &str {
                #tool_name
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_tool_name(attrs: &[syn::Attribute], default_name: &str) -> proc_macro2::TokenStream {
    for attr in attrs {
        if attr.path().is_ident("tool_name") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return quote! { #lit };
            }
        }
    }
    quote! { #default_name }
}
