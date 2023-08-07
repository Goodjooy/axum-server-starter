use darling::util;
use syn::Ident;

use crate::derive_provider::code_gen::{CodeGen, MapToCodeGen};

use super::fields::{FieldInfo, ProviderField};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(provider), supports(struct_named))]
pub struct ProviderDerive {
    ident: Ident,
    #[darling(default)]
    transparent: bool,
    #[darling(default, rename = "r#ref")]
    reference: bool,
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
                .filter_map(|field| field.into_info(self.transparent, self.reference))
                .collect(),
        }
    }
}

pub struct ProviderNeeds {
    pub ident: Ident,
    pub provide: Vec<FieldInfo>,
}

impl ProviderNeeds {
    pub fn to_code_gens(&self) -> (Vec<CodeGen<'_>>, Vec<MapToCodeGen<'_>>) {
        let v = self
            .provide
            .iter()
            .map(|f| CodeGen::new_list(&self.ident, f))
            .fold((Vec::new(), Vec::new()), |(mut cv1, mut cv2), (c1, c2)| {
                cv1.extend(c1);
                cv2.extend(c2);
                (cv1, cv2)
            });
        v
    }
}
