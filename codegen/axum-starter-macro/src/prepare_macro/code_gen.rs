use darling::ToTokens;

use syn::{Lifetime, Type};

use super::inputs::input_fn::InputFn;

pub struct CodeGen<'r> {
    call_async: bool,
    prepare_name: &'r syn::Ident,
    prepare_call: &'r syn::Ident,

    call_args: Vec<&'r Type>,
    args_lifetime: Option<&'r Lifetime>,
}

impl<'r> CodeGen<'r> {
    pub fn new(
        prepare_name: &'r syn::Ident,
        arg_lifetime: &'r Option<Lifetime>,
        InputFn {
            is_async,
            fn_name,
            args_type,
            ..
        }: InputFn<'r>,
    ) -> Self {
        Self {
            call_async: is_async,
            prepare_name,
            prepare_call: fn_name,
            call_args: args_type,
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
            args_lifetime,
        } = self;

        let bound_lifetime = match args_lifetime {
            Some(l) => quote::quote!(#l),
            None => quote::quote!('r),
        };

        let impl_bounds = call_args.iter().map(|ty| {
            quote::quote! {
                Config: for<#bound_lifetime> ::axum_starter::Provider<#bound_lifetime, #ty>,
            }
        });

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
            #[allow(non_snake_case)]
            async fn #prepare_name<Config>(config:std::sync::Arc<Config>) -> impl ::axum_starter::IntoFallibleEffect
            where
                Config : 'static,
                #(#impl_bounds)*
            {
                #prepare_call(
                    #(
                        #args_fetch
                    ),*
                )# awaiting
            }
        };
        tokens.extend(token)
    }
}
