use syn::{parse::Parse, Lifetime, Token};

pub struct PrepareName {
    pub(in crate::prepare_macro) may_fall: bool,
    pub(in crate::prepare_macro) prepare_mode: PrepareFnMode,
    pub(in crate::prepare_macro) ident: syn::Ident,
    pub(in crate::prepare_macro) lt: Option<Lifetime>,
}

#[derive(Debug,Clone,Copy)]
pub enum PrepareFnMode{
    Async,
    AsyncBoxed,
    Sync
}

use syn::custom_keyword;
use syn::spanned::Spanned;

custom_keyword!(sync);

impl Parse for PrepareName {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let need_boxed = if input.peek(Token![box]) {
            input.parse::<Token!(box)>()?;
            true
        } else {
            false
        };

        let sync = if input.peek(sync) {
            input.parse::<sync>()?;
            true
        } else {
            false
        };
        let prepare_mode = match (need_boxed,sync) {
            (true,false)=>PrepareFnMode::AsyncBoxed,
            (false,true)=>PrepareFnMode::Sync,
            (false,false)=>PrepareFnMode::Async,
            (true,true)=> return Err(syn::Error::new(sync.span(), "prepare can only one of `box` or `sync`"))
        };

        let ident = input.parse::<syn::Ident>()?;

        let may_fall = if input.peek(Token![?]) {
            input.parse::<Token!(?)>()?;
            true
        } else {
            false
        };

        let lt = input.parse::<Option<Lifetime>>().unwrap_or_default();
        Ok(Self {
            ident,
            prepare_mode,
            lt,
            may_fall,
        })
    }
}
