use darling::{util::Override, ToTokens};
use syn::{Path, Type};

use super::derive_inputs::{Address, DeriveInput, Logger, Provider};

pub struct ImplAddress<'r> {
    ident: &'r syn::Ident,
    ty: Option<&'r Type>,
    fetcher: Option<&'r Path>,
}

impl<'r> From<&'r DeriveInput> for ImplAddress<'r> {
    fn from(input: &'r DeriveInput) -> Self {
        let ident = &input.ident;
        let (ty, fetcher) = match input.address {
            Address::Provide(Override::Explicit(Provider { ref ty })) => (Some(ty), None),
            Address::Provide(Override::Inherit) => (None, None),
            Address::Func { ref path, ref ty } => (ty.as_ref(), Some(path)),
        };

        Self { ident, ty, fetcher }
    }
}

impl<'r> ToTokens for ImplAddress<'r> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplAddress { ident, ty, fetcher } = self;

        let ty = ty
            .map(|ty| quote::quote!(#ty))
            .unwrap_or_else(|| quote::quote!(::std::net::SocketAddr));

        let fetcher = fetcher
            .map(|fetch| quote::quote!(# fetch ( &self )))
            .unwrap_or_else(|| quote::quote!(::axum_starter::Provider::provide(self)));

        let impl_block = quote::quote! {
            impl ::axum_starter::ServeAddress for #ident{
                type Address = #ty;

                fn get_address(&self) -> Self::Address{
                    #fetcher
                }
            }
        };

        tokens.extend(impl_block)
    }
}

pub struct ImplInitLog<'r> {
    ident: &'r syn::Ident,
    err_type: &'r Type,
    init: &'r Path,
}

impl<'r> From<&'r DeriveInput> for Option<ImplInitLog<'r>> {
    fn from(input: &'r DeriveInput) -> Self {
        let Some(Logger{ func, error }) = input.logger.as_ref() else {return  None;};

        Some(ImplInitLog {
            ident: &input.ident,
            err_type: error,
            init: func,
        })
    }
}

impl<'r> ToTokens for ImplInitLog<'r> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplInitLog {
            err_type,
            init,
            ident,
        } = self;

        let token = quote::quote! {
            impl ::axum_starter::LoggerInitialization for #ident {
                type Error = #err_type;
                fn init_logger(&self) -> Result<(), Self::Error>{
                    #init ( self )
                }
            }
        };

        tokens.extend(token)
    }
}

pub struct ImplServerEffect<'r> {
    ident: &'r syn::Ident,
    func: Option<&'r Path>,
}

impl<'r> From<&'r DeriveInput> for ImplServerEffect<'r> {
    fn from(input: &'r DeriveInput) -> Self {
        Self {
            ident: &input.ident,
            func: match &input.server {
                Override::Inherit => None,
                Override::Explicit(path) => Some(path),
            },
        }
    }
}

impl<'r> ToTokens for ImplServerEffect<'r> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ImplServerEffect { ident, func } = self;

        let effect = func
            .map(|func| quote::quote!(#func (self, server)))
            .unwrap_or_else(|| quote::quote!(server));

        let token = quote::quote! {
            impl ::axum_starter::ConfigureServerEffect for #ident{
                fn effect_server(
                    &self,
                    server: ::axum_starter::Builder<::axum_starter::AddrIncoming>,
                ) -> ::axum_starter::Builder<::axum_starter::AddrIncoming> {
                    #effect
                }
            }
        };

        tokens.extend(token);
    }
}
