use syn::{spanned::Spanned, FnArg, ItemFn, PatType, ReturnType};

pub struct InputFn<'r> {
    pub is_async: bool,
    pub fn_name: &'r syn::Ident,
    pub ret_type: &'r ReturnType,
    pub args_type: Vec<&'r syn::Type>,
}

impl<'r> InputFn<'r> {
   pub fn from_fn_item(item: &'r ItemFn) -> syn::Result<Self> {
        let sig = &item.sig;
        if sig.constness.is_some() {
            Err(syn::Error::new(
                sig.constness.span(),
                "`prepare` not support const fn",
            ))?;
        }

        if sig.unsafety.is_some() {
            Err(syn::Error::new(
                sig.unsafety.span(),
                "`prepare` not support unsafe fn",
            ))?;
        }

        if sig.generics.type_params().next().is_some()
            || sig.generics.const_params().next().is_some()
        {
            Err(syn::Error::new(
                sig.generics.span(),
                "`prepare` not support Type Generic and const Generic",
            ))?;
        }

        if sig.abi.is_some() {
            Err(syn::Error::new(
                sig.abi.span(),
                "`prepare` not support extern fn",
            ))?;
        }

        if let Some(FnArg::Receiver(r)) = sig.inputs.first() {
            Err(syn::Error::new(
                r.span(),
                "`prepare` not support fn with Self receiver",
            ))?;
        }

        Ok(Self {
            is_async: sig.asyncness.is_some(),
            fn_name: &sig.ident,
            ret_type: &sig.output,
            args_type: sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(PatType { ty, .. }) => Some(ty.as_ref()),
                })
                .collect(),
        })
    }
}
