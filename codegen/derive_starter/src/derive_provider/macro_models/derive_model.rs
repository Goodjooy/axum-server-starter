use darling::util;
use syn::Ident;


use crate::derive_provider::code_gen::CodeGen;

use super::fields::{FieldInfo, ProviderField};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(provider), supports(struct_named))]
pub struct ProviderDerive {
    ident: Ident,
    data: darling::ast::Data<util::Ignored, ProviderField>,
}

impl ProviderDerive {
    pub fn into_needs(self) -> ProviderNeeds {
        ProviderNeeds {
            ident: self.ident,
            provide: self
                .data
                .take_struct()
                .unwrap()
                .into_iter()
                .filter_map(ProviderField::into_info)
                .collect(),
        }
    }
}

pub struct ProviderNeeds {
    pub ident: Ident,
    pub provide: Vec<FieldInfo>,
}

impl ProviderNeeds {
    pub fn to_code_gens(&self) -> impl Iterator<Item = CodeGen<'_>> {
        self.provide
            .iter()
            .flat_map(|f| CodeGen::new_list(&self.ident, f))
    }
}
