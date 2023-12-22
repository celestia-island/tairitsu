use proc_macro::TokenStream;
use syn::parse_macro_input;

mod utils;

#[proc_macro]
pub fn register(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as utils::register::DeriveRegisters);
    utils::register::register(input).into()
}
