use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::{visit_lifetime, Visit};
use syn::{punctuated::Punctuated, ItemFn, Lifetime};

use self::{
    code_gen::CodeGen,
    inputs::{attr_name::PrepareName, input_fn::InputFn},
};

pub mod code_gen;
pub mod inputs;

const DEFAULT_LIFETIME_SYMBOL: &str = "'r";

pub fn prepare_macro(
    PrepareName {
        may_fall,
        prepare_mode,
        origin,
        ident,
        lt,
    }: &PrepareName,
    mut item_fn: ItemFn,
) -> syn::Result<proc_macro::TokenStream> {
    if let Some(lt) = lt {
        let mut contain_lifetime = ContainLifetime::new(lt);
        item_fn
            .sig
            .inputs
            .iter()
            .for_each(|arg| contain_lifetime.visit_fn_arg(arg));
        if !item_fn
            .sig
            .generics
            .lifetimes()
            .map(|v| &v.lifetime)
            .any(|l| l == lt)
            && contain_lifetime.1
        {
            item_fn
                .sig
                .generics
                .params
                .push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                    attrs: vec![],
                    lifetime: lt.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                }));
        }
    }

    let input = InputFn::from_fn_item(&item_fn, lt.as_ref())?;
    let code_gen = CodeGen::new(ident, lt, *prepare_mode, *may_fall, input);

    let origin = if *origin {
        quote!(
            #[allow(dead_code)]
            # item_fn
        )
    } else {
        TokenStream::new()
    };
    Ok(quote::quote! {
        # code_gen
        #origin
    }
    .into())
}

struct ContainLifetime<'r>(&'r Lifetime, bool);

impl<'r, 'ast> Visit<'ast> for ContainLifetime<'r> {
    fn visit_lifetime(&mut self, i: &'ast Lifetime) {
        self.1 = self.1 || i == self.0;
        visit_lifetime(self, i);
    }
}

impl<'r> ContainLifetime<'r> {
    fn new(lifetime: &'r Lifetime) -> Self {
        Self(lifetime, false)
    }
}
