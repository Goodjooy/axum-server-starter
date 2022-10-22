use syn::{parse::Parse, Lifetime, Token};

pub struct PrepareName {
    pub(in crate::prepare_macro) need_boxed: bool,
    pub(in crate::prepare_macro) ident: syn::Ident,
    pub(in crate::prepare_macro) lt: Option<Lifetime>,
}

impl Parse for PrepareName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let need_boxed = if input.peek(Token![box]) {
            input.parse::<Token!(box)>()?;
            true
        } else {
            false
        };
        let ident = input.parse::<syn::Ident>()?;

        let lt = input.parse::<Option<Lifetime>>().unwrap_or_default();
        Ok(Self {
            need_boxed,
            ident,
            lt,
        })
    }
}
