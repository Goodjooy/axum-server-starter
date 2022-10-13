mod derive_provider;
mod prepare_macro;
use prepare_macro::inputs::attr_name::PrepareName;
use syn::{parse_macro_input, DeriveInput, ItemFn};

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
    let prepare_name = parse_macro_input!(attrs as PrepareName);
    let fn_item = parse_macro_input!(input as ItemFn);

    match prepare_macro::prepare_macro(&prepare_name, fn_item) {
        Ok(token) => token,
        Err(error) => error.to_compile_error().into(),
    }
}
