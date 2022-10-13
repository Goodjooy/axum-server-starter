use darling::FromDeriveInput;
use syn::DeriveInput;

use self::marco_models::derive_model::ProviderDerive;

mod code_gen;
mod marco_models;

pub fn provider_derive(derive_input: DeriveInput) -> darling::Result<proc_macro::TokenStream> {
    if !derive_input.generics.params.is_empty() {
        Err(darling::Error::unexpected_type("Unsupported Generic")
            .with_span(&derive_input.generics))?;
    }

    let provider =
        <ProviderDerive as FromDeriveInput>::from_derive_input(&derive_input)?.into_needs();

    let code_gen = provider.to_code_gens();

    Ok(quote::quote! {
        #(#code_gen)*
    }
    .into())
}
