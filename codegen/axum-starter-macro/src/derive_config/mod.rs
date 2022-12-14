use darling::FromDeriveInput;
use syn::DeriveInput;

use self::code_gen::{ImplAddress, ImplInitLog, ImplServerEffect};

mod code_gen;
pub mod derive_inputs;

pub fn provider_derive(derive_input: DeriveInput) -> darling::Result<proc_macro::TokenStream> {
    if !derive_input.generics.params.is_empty() {
        Err(darling::Error::unexpected_type("Unsupported Generic")
            .with_span(&derive_input.generics))?;
    }

    let config =
        <self::derive_inputs::DeriveInput as FromDeriveInput>::from_derive_input(&derive_input)?;

    let address = ImplAddress::from(&config);
    let logger = Option::<ImplInitLog>::from(&config);
    let server = ImplServerEffect::from(&config);
    Ok(quote::quote! {
        #address
        #logger
        #server
    }
    .into())
}
