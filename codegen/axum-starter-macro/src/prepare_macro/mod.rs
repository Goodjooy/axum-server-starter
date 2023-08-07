use syn::{punctuated::Punctuated, ItemFn};

use self::{
    code_gen::CodeGen,
    inputs::{attr_name::PrepareName, input_fn::InputFn},
};

pub mod code_gen;
pub mod inputs;

pub fn prepare_macro(
    PrepareName {
        may_fall,
        prepare_mode,
        ident,
        lt,
    }: &PrepareName,
    mut item_fn: ItemFn,
) -> syn::Result<proc_macro::TokenStream> {
    if let Some(lt) = lt {
        let mut contain_lifetime = ContainLifetime::new(lt);
        item_fn.sig.inputs.iter().for_each(|arg| contain_lifetime.visit_fn_arg(arg));
        if !item_fn.sig.generics.lifetimes().map(|v| &v.lifetime).any(|l| l == lt) && contain_lifetime.1 {
            item_fn.sig.generics.params.push(syn::GenericParam::Lifetime(syn::LifetimeParam {
                attrs: vec![],
                lifetime: lt.clone(),
                colon_token: None,
                bounds: Punctuated::new(),
            }));
        }
    }

    let input = InputFn::from_fn_item(&item_fn, lt.as_ref())?;
    let code_gen = CodeGen::new(ident, lt, *prepare_mode, *may_fall, input);

    Ok(quote::quote! {
        # code_gen
        #[allow(clippy::needless_lifetimes)]
        #[allow(dead_code)]
        # item_fn
    }
}

impl<'r> ContainLifetime<'r> {
    fn new(lifetime: &'r Lifetime) -> Self {
        Self(lifetime, false)
    }
}