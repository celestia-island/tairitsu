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

fn expand_component_impl(input: ItemFn) -> Result<TokenStream2> {
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_return = match &input.sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        _ => syn::parse_quote! { tairitsu_vdom::VNode },
    };

    let mut fields = Vec::new();
    let mut field_defaults = Vec::new();
    let mut builder_methods = Vec::new();
    let mut prop_names = Vec::new();

    for arg in &input.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
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

    let props_name = format_ident!("{}Props", to_pascal_case(&fn_name.to_string()));
    let builder_name = format_ident!("{}Builder", to_pascal_case(&fn_name.to_string()));

    let props_struct = quote! {
        #[derive(Debug, Clone)]
        #fn_vis struct #props_name {
            #(#fields),*
        }
    };

    let builder_struct = quote! {
        #[derive(Debug, Clone)]
        #fn_vis struct #builder_name{
            #(#fields),*
        }
    };

    let props_impl = quote! {
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
    };

    let builder_impl = quote! {
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
    };

    let original_fn = quote! {
        #fn_vis fn #fn_name(props: #props_name) #fn_return {
            let #props_name { #(#prop_names),* } = props;
            #fn_block
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
