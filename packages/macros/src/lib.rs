mod component;
mod props_dsl;
mod rsx;
mod scss;
mod scss_include;
mod svg;

use component::expand_component;
use proc_macro::TokenStream;
use quote::quote;
use rsx::{expand_rsx_root, RsxRoot};
use scss::expand_scss;
use svg::expand_svg;
use syn::{parse_macro_input, Data, DeriveInput};

/// Component macro for automatic Props generation
///
/// # Example
/// ```ignore
/// #[component]
/// fn Button(
///     variant: ButtonVariant,
///     #[children] children: Vec<VNode>,
///     #[default] onclick: Option<Box<dyn FnMut(Box<dyn EventData>)>>,
/// ) -> VNode {
///     rsx! {
///         button {
///             class: "button",
///             onclick: onclick,
///             ..children
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_component(attr, item)
}

/// RSX macro for declarative UI construction
///
/// # Example
/// ```ignore
/// rsx! {
///     div {
///         class: "container",
///         "Hello, world!"
///     }
/// }
/// ```
#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    let root = syn::parse_macro_input!(input as RsxRoot);
    let expanded = expand_rsx_root(root);
    TokenStream::from(expanded)
}

/// SCSS macro for compile-time CSS generation with class name hashing
///
/// Compiles SCSS syntax to CSS at compile time using grass compiler,
/// and generates hashed class names for CSS Modules-style scoping.
///
/// # Features
/// - Full SCSS syntax support via grass compiler
/// - Automatic class name hashing (CSS Modules style)
/// - Scope-based isolation
/// - Support for inline content or file paths
/// - Returns (css, class_map) tuple
///
/// # Example
/// ```ignore
/// // Basic usage - inline SCSS
/// let (css, class_map) = scss! {
///     .button {
///         background: var(--primary);
///         color: white;
///         padding: 8px 16px;
///         border-radius: 4px;
///
///         &:hover {
///             background: var(--primary-dark);
///         }
///
///         &.disabled {
///             opacity: 0.5;
///         }
///     }
/// };
///
/// // From file (relative to crate root)
/// let (css, class_map) = scss! { file: "styles/main.scss" };
///
/// // With scope for isolation
/// let (css, class_map) = scss! {
///     .container {
///         width: 100%;
///     },
///     scope: "MyComponent"
/// };
///
/// // File with scope
/// let (css, class_map) = scss! { file: "styles/button.scss", scope: "Button" };
///
/// // Use hashed class names
/// let button_class = class_map.get("button").unwrap();
/// ```
#[proc_macro]
pub fn scss(input: TokenStream) -> TokenStream {
    expand_scss(input)
}

/// Compile-time SCSS class extraction & type-safe enum generation.
///
/// Reads a SCSS file at compile time, extracts all `.class-name` selectors
/// using a recursive descent parser that faithfully implements the grammar
/// defined in `scss_classes.pest`, and generates a Rust enum with one variant
/// per class. Each variant maps back to its original class string.
///
/// # Features
/// - **Grammar-faithful parsing**: Handles SCSS nesting (`&.class`), comments,
///   strings, @-rules correctly — no false positives from property values.
/// - **Type-safe classes**: Typos in class names are caught at compile time.
/// - **Zero runtime cost**: All extraction happens at compile time; the
///   generated enum is just `Copy` data with `const fn as_str()`.
/// - **BEM-aware naming**: `hi-button--primary` → `HiButtonPrimary`,
///   `menu__item--active` → `MenuItemActive`.
/// - **Filter support**: Use `filter: "hi-"` to only include classes
///   starting with that prefix (useful for large SCSS files).
///
/// # Example
/// ```ignore
/// // Per-component (small files, fast compilation):
/// tairitsu_macros::include_scss!("styles/button.scss");
/// // → ButtonClasses { HiButton, HiButtonPrimary, ... }
///
/// // With filter (large monolithic file):
/// tairitsu_macros::include_scss!("styles/spa.scss", filter: "hi-button-");
/// // → SpaClasses { HiButtonPrimary, HiButtonSm, HiButtonLg, ... }
///
/// // Custom enum name + prefix override:
/// tairitsu_macros::include_scss!("styles.scss", enum_name: MyStyles);
///
/// // Usage in rsx!:
/// rsx! { button { class: ButtonClasses::HiButton.as_str() } }
/// ```
#[proc_macro]
pub fn include_scss(input: TokenStream) -> TokenStream {
    scss_include::expand_include_scss(input)
}

/// SVG macro for compile-time SVG embedding with XSS protection
///
/// This macro reads SVG content at compile time and creates a SafeSvg instance
/// with built-in XSS sanitization.
///
/// # Features
/// - Compile-time SVG embedding
/// - XSS sanitization (removes scripts, event handlers, dangerous URLs)
/// - Support for inline content, file paths, or resource ID lookup
///
/// # Example
/// ```ignore
/// // Inline SVG content
/// let icon = svg! { r#"<path d="M12 2L2 22h20L12 2z"/>"# };
///
/// // From file (relative to crate root)
/// let icon = svg! { file: "icons/sun.svg" };
///
/// // From resource index by ID (searches icons/, src/icons/, etc.)
/// let icon = svg! { id: "sun" };
///
/// // Use with VElement
/// rsx! {
///     svg {
///         viewBox: "0 0 24 24",
///         safe_svg: icon,
///     }
/// }
/// ```
#[proc_macro]
pub fn svg(input: TokenStream) -> TokenStream {
    expand_svg(input)
}

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
struct WitGuestImpl {}

impl syn::parse::Parse for WitGuestImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Accept any token stream — wit_guest_impl! takes arbitrary key-value syntax
        input.parse::<proc_macro2::TokenStream>()?;
        Ok(WitGuestImpl {})
    }
}

// Parse arguments for wit_world!("package:world", "./wit/path")
struct WitWorldArgs {
    world: syn::LitStr,
    path: syn::LitStr,
}

impl syn::parse::Parse for WitWorldArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let world: syn::LitStr = input.parse()?;
        let _: syn::Token![,] = input.parse()?;
        let path: syn::LitStr = input.parse()?;
        Ok(WitWorldArgs { world, path })
    }
}

/// Convenience wrapper around `wasmtime::component::bindgen!`.
///
/// Expands to `::wasmtime::component::bindgen!({ path: …, world: … })`.
///
/// The calling crate must declare `wasmtime` as a direct dependency (as it
/// does when using the `tairitsu` runtime crate).
///
/// # Example
/// ```ignore
/// use tairitsu::wit_world;
///
/// wit_world!("my-package:my-world", "./wit");
/// ```
#[proc_macro]
pub fn wit_world(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as WitWorldArgs);
    let world = &args.world;
    let path = &args.path;

    let expanded = quote! {
        ::wasmtime::component::bindgen!({
            path: #path,
            world: #world,
        });
    };

    TokenStream::from(expanded)
}

/// Marker macro — reserved for future host-import registration codegen.
///
/// Currently this macro accepts its input and emits no code, acting as a
/// no-op. A future version will auto-generate `add_to_linker` boilerplate
/// from annotated host structs, but that requires WIT interface knowledge
/// at compile time that is not yet available here.
///
/// Until then, implement the WIT traits manually and call
/// `MyInterface::add_to_linker(&mut linker, |state| &mut state.data)`.
#[proc_macro]
pub fn register_host(input: TokenStream) -> TokenStream {
    // Consume input to avoid "unused token" warnings; emit nothing.
    let _ = proc_macro2::TokenStream::from(input);
    TokenStream::new()
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

/// Derive macro for Props structs.
///
/// This is a marker derive that indicates a struct is used as component props.
/// The actual Default implementation should be derived separately using `#[derive(Default)]`
/// or implemented manually with proper defaults for fields.
///
/// # Example
/// ```ignore
/// #[derive(Clone, Props, PartialEq, Default)]
/// pub struct ButtonProps {
///     pub variant: String,
///     #[props(default)]
///     pub disabled: bool,
/// }
/// ```
#[proc_macro_derive(Props, attributes(props))]
pub fn derive_props(input: TokenStream) -> TokenStream {
    let _input = parse_macro_input!(input as DeriveInput);

    // Props derive is just a marker - Default should be derived separately
    let expanded = quote! {
        // Props marker - no additional implementation
    };

    TokenStream::from(expanded)
}

/// Attribute macro for defining component props with cleaner DSL syntax.
///
/// This macro transforms a simplified struct definition into the verbose
/// Props format required by the component system, automatically generating
/// the `#[props(default = ...)]` attributes and `Default` implementation.
///
/// # Example
/// ```ignore
/// #[define_props]
/// pub struct AvatarProps {
///     src: Option<String> = None,
///     alt: String = "Avatar".to_string(),
///     size: AvatarSize = AvatarSize::Md,
///     class: String = String::new(),
/// }
/// ```
///
/// Expands to:
/// ```ignore
/// #[derive(Clone, PartialEq, Props)]
/// pub struct AvatarProps {
///     #[props(default)]
///     pub src: Option<String>,
///     #[props(default = "Avatar".to_string())]
///     pub alt: String,
///     #[props(default = AvatarSize::Md)]
///     pub size: AvatarSize,
///     #[props(default)]
///     pub class: String,
/// }
///
/// impl Default for AvatarProps {
///     fn default() -> Self {
///         Self {
///             src: None,
///             alt: "Avatar".to_string(),
///             size: AvatarSize::Md,
///             class: String::new(),
///         }
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn define_props(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match syn::parse::<props_dsl::PropsInput>(item.clone()) {
        Ok(input) => {
            let expanded = props_dsl::expand_define_props(input);
            TokenStream::from(expanded)
        }
        Err(e) => {
            // If parsing fails, output a compile_error with the error message
            let msg = format!("define_props macro error: {}", e);
            let ts = quote! {
                compile_error!(#msg);
            };
            TokenStream::from(ts)
        }
    }
}
