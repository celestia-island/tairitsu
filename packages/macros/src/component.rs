use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Attribute, FnArg, Ident, ItemFn, Pat, PatType, Result};

pub fn expand_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

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

    // Strip doc comments from function parameters and extract info
    for arg in &mut input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            // Remove doc comments from parameter attributes
            pat_type.attrs.retain(|attr| {
                !attr.path().is_ident("doc")
            });

            // Skip extraction if using existing props
            if !uses_existing_props {
                let (name, has_default, _is_children) = extract_arg_info(pat_type)?;
                let ty = (*pat_type.ty).clone();

                prop_names.push(name.clone());

                fields.push(quote! {
                    pub #name: #ty
                });

                if has_default {
                    field_defaults.push(quote! {
                        #name: Default::default()
                    });
                } else {
                    field_defaults.push(quote! {
                        #name: std::marker::PhantomData::<()>.into()
                    });
                }

                builder_methods.push(quote! {
                    pub fn #name(mut self, #name: #ty) -> Self {
                        self.#name = #name;
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
            pat_type.attrs.retain(|attr| {
                !attr.path().is_ident("doc")
            });
            cleaned_inputs.push(FnArg::Typed(pat_type));
        } else {
            cleaned_inputs.push(arg.clone());
        }
    }

    // Create the function
    let original_fn = if uses_existing_props {
        // Use the existing Props type
        let props_type = existing_props_name.unwrap();
        quote! {
            #fn_vis fn #fn_name(props: #props_type) #fn_return {
                #fn_block
            }
        }
    } else {
        quote! {
            #fn_vis fn #fn_name(props: #props_name) #fn_return {
                let #props_name { #(#prop_names),* } = props;
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

    let has_default = has_attribute(&pat_type.attrs, "default");
    let is_children = has_attribute(&pat_type.attrs, "children");

    Ok((name, has_default, is_children))
}

fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| {
        if let Some(ident) = attr.path().get_ident() {
            ident == name
        } else {
            false
        }
    })
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in s.chars() {
        if ch == '_' || ch == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(ch.to_uppercase().next().unwrap());
            capitalize_next = false;
        } else {
            result.push(ch);
        }
    }

    result
}
