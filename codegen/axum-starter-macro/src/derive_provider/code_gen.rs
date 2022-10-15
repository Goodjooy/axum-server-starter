use darling::ToTokens;
use syn::{Ident, Type};

use super::macro_models::fields::{FieldInfo, ProvideType};

pub struct CodeGen<'i> {
    provider: &'i Ident,
    provide_type: ProvideType,
    field_name: &'i Ident,
    field_ty: &'i Type,
    wrap: Option<&'i Ident>,
}

impl<'i> CodeGen<'i> {
    pub fn new_list(provider: &'i Ident, info: &'i FieldInfo) -> impl Iterator<Item = CodeGen<'i>> {
        Some(Self {
            provider,
            field_name: &info.src_field_ident,
            field_ty: &info.ty,
            wrap: info.wrapper_name.as_ref(),
            provide_type: info.provide_type,
        })
        .into_iter()
        .chain(info.aliases.iter().map(|wrap| CodeGen {
            provider,
            field_name: &info.src_field_ident,
            field_ty: &info.ty,
            wrap: Some(wrap),
            provide_type: info.provide_type,
        }))
    }
}

impl<'i> ToTokens for CodeGen<'i> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = self.field_ty;
        if let Some(wrap) = self.wrap {
            let token = match self.provide_type {
                ProvideType::Ref => {
                    quote::quote! {
                        pub struct #wrap <'r> (pub &'r #ty);
                    }
                }
                ProvideType::Owned => {
                    quote::quote! {
                        pub struct #wrap (pub  #ty);
                    }
                }
            };
            tokens.extend(token);
        }

        let ty = self.field_ty;
        let provide_type = &match (self.wrap, self.provide_type) {
            (None, ProvideType::Ref) => quote::quote!(& 'r #ty),
            (None, ProvideType::Owned) => quote::quote!( #ty),
            (Some(wrap), ProvideType::Ref) => quote::quote! {#wrap<'r>},
            (Some(wrap), ProvideType::Owned) => quote::quote! {#wrap},
        };

        let field_name = self.field_name;
        let fetch = match (self.wrap, self.provide_type) {
            (None, ProvideType::Ref) => quote::quote! {&self.#field_name},
            (None, ProvideType::Owned) => {
                quote::quote! { std::clone::Clone::clone(&self.#field_name) }
            }
            (Some(wrap), ProvideType::Ref) => quote::quote! {#wrap ( &self.#field_name )},
            (Some(wrap), ProvideType::Owned) => {
                quote::quote! {#wrap ( std::clone::Clone::clone(&self.#field_name) )}
            }
        };
        let ty = self.field_ty;
        let bound = match self.provide_type {
            ProvideType::Ref => quote::quote!(),
            ProvideType::Owned => quote::quote! {where #ty : std::clone::Clone },
        };
        let this = self.provider;

        let token = quote::quote! {
            impl<'r> ::axum_starter::Provider<'r, #provide_type> for #this #bound{
                fn provide(&'r self) -> #provide_type{
                    # fetch
                }
            }
        };

        tokens.extend(token)
    }
}
