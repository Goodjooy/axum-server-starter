use darling::FromDeriveInput;
use syn::{spanned::Spanned, DeriveInput};

use self::{
    code_gen::GenImplFromState,
    input::{StateField, StateInput},
};

pub mod code_gen;
pub mod input;

pub fn from_state_collector_macro(input: DeriveInput) -> darling::Result<proc_macro::TokenStream> {
    if !input.generics.params.is_empty() {
        Err(
            darling::error::Error::unexpected_type("Not support generic type")
                .with_span(&input.generics.span()),
        )?;
    }

    let input: StateInput = FromDeriveInput::from_derive_input(&input)?;

    let code_gen = GenImplFromState {
        ident: &input.ident,
        fields: input
            .data
            .take_struct()
            .ok_or_else(|| syn::Error::new(input.ident.span(), "Expect Struct, but get Enum"))?
            .fields
            .into_iter()
            .map(|StateField { ident, ty }| (ident, ty))
            .collect(),
    };

    Ok(quote::quote!(#code_gen).into())
}
