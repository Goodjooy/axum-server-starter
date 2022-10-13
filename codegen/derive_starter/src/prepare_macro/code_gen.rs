use darling::ToTokens;

use syn::{Lifetime, ReturnType, Type};

use super::inputs::input_fn::InputFn;

pub struct CodeGen<'r> {
    call_async: bool,
    prepare_name: &'r syn::Ident,
    prepare_call: &'r syn::Ident,

    call_args: Vec<&'r Type>,
    args_lifetime: Option<&'r Lifetime>,
    call_ret: &'r ReturnType,
}

impl<'r> CodeGen<'r> {
    pub fn new(
        prepare_name: &'r syn::Ident,
        arg_lifetime: &'r Option<Lifetime>,
        InputFn {
            is_async,
            fn_name,
            ret_type,
            args_type,
        }: InputFn<'r>,
    ) -> Self {
        Self {
            call_async: is_async,
            prepare_name,
            prepare_call: fn_name,
            call_args: args_type,
            call_ret: ret_type,
            args_lifetime: arg_lifetime.as_ref(),
        }
    }
}

impl<'r> ToTokens for CodeGen<'r> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            call_async,
            prepare_name,
            prepare_call,
            call_args,
            call_ret,
            args_lifetime,
        } = self;

        // prepare type
        let token = quote::quote! {
            pub struct #prepare_name;
        };

        tokens.extend(token);

        let bound_lifetime = match args_lifetime {
            Some(l) => quote::quote!(#l),
            None => quote::quote!('r),
        };

        let impl_bounds = call_args.iter().map(|ty| {
            quote::quote! {
                Config: for<#bound_lifetime> ::axum_starter::Provider<#bound_lifetime, #ty>,
            }
        });

        let ret_bound = match call_ret {
            ReturnType::Default => quote::quote! {
                () : ::axum_starter::IntoFallibleEffect + 'static,
            },
            ReturnType::Type(_, ty) => quote::quote! {
                #ty : ::axum_starter::IntoFallibleEffect + 'static,
            },
        };

        let args_fetch = call_args.iter().map(|_ty| {
            quote::quote! {
                ::axum_starter::Provider::provide(std::ops::Deref::deref(&config))
            }
        });

        let awaiting = if *call_async {
            Some(quote::quote!(.await))
        } else {
            None
        };

        // impl prepare
        let token = quote::quote! {
            impl<Config> ::axum_starter::Prepare<Config> for #prepare_name
            where
                Config : 'static,
                #(#impl_bounds)*
                #ret_bound
            {
                fn prepare(self, config: std::sync::Arc<Config>) -> ::axum_starter::BoxPreparedEffect{
                    use std::boxed::Box;
                    Box::pin(async move {
                        let ret = #prepare_call(
                            #(
                                #args_fetch
                            ),*
                        )# awaiting;

                        ::axum_starter::IntoFallibleEffect::into_effect(ret)
                        .map(|effect| Box::new(effect) as Box<dyn ::axum_starter::PreparedEffect>)
                        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
                    })
                }
            }
        };
        tokens.extend(token)
    }
}
