mod code_gen;
use marco_models::derive_model::ProviderDerive;
use syn::{parse_macro_input, DeriveInput};

mod marco_models;
#[macro_use]
mod utils;

use darling::FromDeriveInput;
#[proc_macro_derive(Provider, attributes(provider))]
pub fn derive_config_provider(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    if !derive_input.generics.params.is_empty() {
        return darling::Error::unexpected_type("Unsupported Generic")
            .with_span(&derive_input.generics)
            .write_errors()
            .into();
    }

    let provider = darling_err!(<ProviderDerive as FromDeriveInput>::from_derive_input(
        &derive_input
    ))
    .into_needs();

    let code_gen = provider.to_code_gens();

    quote::quote! {
        #(#code_gen)*
    }
    .into()
}
