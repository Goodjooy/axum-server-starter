use darling::ToTokens;

use quote::format_ident;
use syn::{Lifetime, Stmt, Type};

use super::inputs::input_fn::{ArgInfo, GenericWithBound, InputFn};

pub struct CodeGen<'r> {
    may_fall: bool,
    boxed: bool,

    prepare_name: &'r syn::Ident,
    prepare_generic: GenericWithBound<'r>,

    call_args: Vec<ArgInfo<'r>>,
    fn_body: &'r [Stmt],
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
            args_type,
            generic,
            ret,
            fn_body,
        }: InputFn<'r>,
    ) -> Self {
        Self {
            prepare_name,
            call_args: args_type,
            args_lifetime: arg_lifetime.as_ref(),
            prepare_generic: generic,
            boxed,
            may_fall,
            ret_type: ret,
            fn_body,
        }
    }
}

impl<'r> ToTokens for CodeGen<'r> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            prepare_name,
            call_args,
            args_lifetime,
            prepare_generic,
            boxed,
            may_fall,
            ret_type,
            fn_body,
            ..
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

        let extra_bounds = prepare_generic.where_closure.as_ref();

        let impl_bounds = call_args.iter().map(|ArgInfo { ty, .. }| {
            quote::quote! {
                Config: for<#bound_lifetime> ::axum_starter::Provider<#bound_lifetime, #ty>,
            }
        });

        let args_fetch = call_args.iter().map(|_ty| {
            quote::quote! {
                ::axum_starter::Provider::provide(::core::ops::Deref::deref(&config))
            }
        });
        let inner_struct_name = format_ident!("__InnerArgsStruct");
        let inner_struct = {
            let (_, _, where_clause) = prepare_generic.origin.split_for_impl();
            let ty_generic = prepare_generic.type_generic.iter();
            let types = call_args.iter().map(|ArgInfo { ty, .. }| ty);
            let phantom = prepare_generic.type_generic.iter().map(|tp|&tp.ident);

            quote::quote!(struct #inner_struct_name <#bound_lifetime, #(#ty_generic,)*> ( #(#types,)* core::marker::PhantomData<& #bound_lifetime (#(#phantom),*)> ) #where_clause;)
        };

        let construct_inner_struct = {
            let ty_generic = prepare_generic.type_generic.iter().map(|tp|&tp.ident);
            quote::quote!(let args = #inner_struct_name::<#(#ty_generic),*>(#(#args_fetch,)* core::marker::PhantomData);)
        };

        let fetch_args = {
            let pattens = call_args.iter().map(|ArgInfo { patten, .. }| patten);

            quote::quote!(let #inner_struct_name(#(#pattens,)* _) = args;)
        };

        let execute_prepare = quote::quote! {
            #inner_struct
            #construct_inner_struct
            #fetch_args
        };

        let func_call = quote::quote! {
            {
                #(#fn_body)*
            }
        };

        let mapped_func_call = if *may_fall {
            func_call
        } else {
            quote::quote!(
                let ret = #func_call;
                ::core::result::Result::Ok(
                    ret
                )
            )
        };

        let async_boxed = if *boxed {
            quote::quote! {
                ::std::boxed::Box::pin(
                    async move {
                        #execute_prepare
                        #mapped_func_call
                    }
                )
            }
        } else {
            quote::quote! {
                #execute_prepare
                #mapped_func_call

            }
        };

        // ret type
        let ret_type = match ret_type {
            Some(ty) => quote::quote!(#ty),
            None => quote::quote!(()),
        };

        let ret_type = if *may_fall {
            ret_type
        } else {
            quote::quote!(
                ::core::result::Result<
                    #ret_type ,
                    ::core::convert::Infallible
                >
            )
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
                config: ::std::sync::Arc<Config>
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
