use syn::{parse::Parse, Lifetime};

pub struct PrepareName(pub(in crate::prepare_macro) syn::Ident,pub Option<Lifetime>);

impl Parse for PrepareName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?,input.parse().unwrap_or_default()))
    }
}
