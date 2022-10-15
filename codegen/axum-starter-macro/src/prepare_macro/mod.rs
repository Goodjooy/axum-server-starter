use syn::{punctuated::Punctuated, ItemFn};

use self::{
    code_gen::CodeGen,
    inputs::{attr_name::PrepareName, input_fn::InputFn},
};

pub mod code_gen;
pub mod inputs;

pub fn prepare_macro(
    PrepareName(name, lt): &PrepareName,
    mut item_fn: ItemFn,
) -> syn::Result<proc_macro::TokenStream> {
    if let Some(lt) = lt {
        if !item_fn
            .sig
            .generics
            .lifetimes()
            .map(|v| &v.lifetime)
            .any(|l| l == lt)
        {
            item_fn
                .sig
                .generics
                .params
                .push(syn::GenericParam::Lifetime(syn::LifetimeDef {
                    attrs: vec![],
                    lifetime: lt.clone(),
                    colon_token: None,
                    bounds: Punctuated::new(),
                }));
        }
    }

    let input = InputFn::from_fn_item(&item_fn)?;
    let code_gen = CodeGen::new(name, lt, input);

    Ok(quote::quote! {
        # code_gen
        # item_fn
    }
    .into())
}