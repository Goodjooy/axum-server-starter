use darling::{ast::Data, util::Ignored};
use syn::Type;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(supports(struct_named, struct_tuple, struct_unit))]
pub struct StateInput {
    pub ident: syn::Ident,
    pub data: Data<Ignored, StateField>,
}

#[derive(Debug, darling::FromField)]
pub struct StateField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}
