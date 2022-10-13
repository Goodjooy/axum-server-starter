use syn::parse::Parse;

pub struct PrepareName(pub(in crate::prepare_macro) syn::Ident);

impl Parse for PrepareName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}
