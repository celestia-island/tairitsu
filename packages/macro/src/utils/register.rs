use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};

pub struct DeriveRegisters {}

impl Parse for DeriveRegisters {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let TokenStream { .. } = input.parse()?;

        Ok(Self {})
    }
}

pub fn register(input: DeriveRegisters) -> TokenStream {
    let DeriveRegisters { .. } = &input;

    quote! {}
}
