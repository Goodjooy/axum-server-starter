use syn::ItemFn;

use self::{inputs::{attr_name::PrepareName, input_fn::InputFn}, code_gen::CodeGen};

pub mod code_gen;
pub mod inputs;

pub fn prepare_macro(
    PrepareName(name): &PrepareName,
    item_fn: &ItemFn,
) -> syn::Result<proc_macro::TokenStream> {
    let input = InputFn::from_fn_item(item_fn)?;
    let code_gen = CodeGen::new(name,input);


    Ok(quote::quote!{
        # code_gen
        # item_fn
    }.into())

}
