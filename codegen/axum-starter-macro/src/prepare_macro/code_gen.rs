use darling::ToTokens;

use syn::{punctuated::Punctuated, Lifetime, Token, Type};

use super::inputs::input_fn::{GenericWithBound, InputFn};

pub struct CodeGen<'r> {
    call_async: bool,
    may_fall: bool,
    boxed: bool,
    prepare_name: &'r syn::Ident,
    prepare_generic: GenericWithBound<'r>,
    prepare_call: &'r syn::Ident,

    call_args: Vec<&'r Type>,
    args_lifetime: Option<&'r Lifetime>,
    ret_type: Option<&'r Type>,
}

impl<'r> CodeGen<'r> {
    pub fn new(
        prepare_name: &'r syn::Ident,
        arg_lifetime: &'r Option<Lifetime>,
        boxed: bool,
        may_fall: bool,
        InputFn {
            is_async,
            fn_name,
            args_type,
            generic,
            ret,
        }: InputFn<'r>,
    ) -> Self {
        Self {
            call_async: is_async,
            prepare_name,
            prepare_call: fn_name,
            call_args: args_type,
            args_lifetime: arg_lifetime.as_ref(),
            prepare_generic: generic,
            boxed,
            may_fall,
            ret_type: ret,
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
            args_lifetime,
            prepare_generic,
            boxed,
            may_fall,
            ret_type,
        } = self;

        let bound_lifetime = match args_lifetime {
            Some(l) => quote::quote!(#l),
            None => quote::quote!('r),
        };

        let extra_generic = {
            let GenericWithBound {
                type_generic,
                const_generic,
                ..
            } = prepare_generic;
            quote::quote! {
                # type_generic
                # const_generic
            }
        };

        let generic_set = {
            let GenericWithBound {
                type_generic,
                const_generic,
                ..
            } = prepare_generic;

            let ty_generic = type_generic
                .into_iter()
                .map(|v| &v.ident)
                .collect::<Punctuated<_, Token!(,)>>();
            let const_generic = const_generic
                .into_iter()
                .map(|v| &v.ident)
                .collect::<Punctuated<_, Token!(,)>>();

            quote::quote!(
                #ty_generic
                #const_generic
            )
        };

        let extra_bounds = prepare_generic.where_closure.as_ref();

        let impl_bounds = call_args.iter().map(|ty| {
            quote::quote! {
                Config: for<#bound_lifetime> ::axum_starter::Provider<#bound_lifetime, #ty>,
            }
        });

        let args_fetch = call_args.iter().map(|_ty| {
            quote::quote! {
                ::axum_starter::Provider::provide(::core::ops::Deref::deref(&config))
            }
        });

        let awaiting = if *call_async {
            Some(quote::quote!(.await))
        } else {
            None
        };

        let func_call = quote::quote! {
            #prepare_call::<
                #generic_set
                >(
                    #(
                        #args_fetch
                    ),*
                )#awaiting
        };

        let mapped_func_call = if *may_fall {
            func_call
        } else {
            quote::quote!(
                ::core::result::Result::Ok(
                    #func_call
                )
            )
        };

        let async_boxed = if *boxed {
            quote::quote! {
                ::std::boxed::Box::pin(
                    async move {
                        #mapped_func_call
                    }
                )
            }
        } else {
            mapped_func_call
        };

        // ret type
        let ret_type = match ret_type {
            Some(ty) => quote::quote!(#ty),
            None => quote::quote!(()),
        };

        let ret_type = if *may_fall {
            quote::quote!(
                ::core::result::Result<
                    <#ret_type as ::axum_starter::FalliblePrepare>::Effect,
                    <#ret_type as ::axum_starter::FalliblePrepare>::Error,

                >
            )
        } else {
            quote::quote!(::core::result::Result <#ret_type ,::core::convert::Infallible>)
        };

        let boxed_ret = if *boxed {
            quote::quote!(
                ::std::pin::Pin<
                    ::std::boxed::Box<
                        dyn ::core::future::Future<Output = #ret_type>,
                    >,
                >
            )
        } else {
            quote::quote!(#ret_type)
        };

        let boxed_async_signal = if *boxed {
            None
        } else {
            Some(quote::quote!(async))
        };

        // impl prepare
        let token = quote::quote! {
            #[allow(non_snake_case)]
            pub #boxed_async_signal fn #prepare_name<
                Config,
                #extra_generic
                >
            (
                config:std::sync::Arc<Config>
            ) -> #boxed_ret
            where
                Config : 'static,
                #(#impl_bounds)*
                #extra_bounds
            {
                #async_boxed
            }
        };

        tokens.extend(token)
    }
}
