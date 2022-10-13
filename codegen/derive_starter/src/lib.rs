mod derive_provider;
use syn::{parse_macro_input, DeriveInput};

#[macro_use]
mod utils;

#[proc_macro_derive(Provider, attributes(provider))]
pub fn derive_config_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    darling_err!(derive_provider::provider_derive(derive_input))
}
#[proc_macro_attribute]
pub fn prepare(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    todo!()
}
