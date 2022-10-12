use darling::ToTokens;
use syn::{Ident, Type};

use crate::marco_models::fields::FieldInfo;

pub struct CodeGen<'i> {
    provider: &'i Ident,
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
            wrap: if info.transparent {
                Some(&info.wrapper_name)
            } else {
                None
            },
        })
        .into_iter()
        .chain(info.aliases.iter().map(|wrap| CodeGen {
            provider,
            field_name: &info.src_field_ident,
            field_ty: &info.ty,
            wrap: Some(wrap),
        }))
    }
}

impl<'i> ToTokens for CodeGen<'i> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(wrap) = self.wrap {
            let ty = self.field_ty;
            let token = quote::quote! {
                pub struct #wrap (#ty);
            };
            tokens.extend(token);
        }
        let ty = self.field_ty;
        let provide_type = &if let Some(wrap) = self.wrap {
            quote::quote! {#wrap}
        } else {
            quote::quote!(#ty)
        };
        let field_name = self.field_name;
        let fetch = if let Some(wrap) = self.wrap {
            quote::quote! {
                #wrap ( self.#field_name )
            }
        } else {
            quote::quote! {
                self.#field_name
            }
        };

        let this = self.provider;

        let token = quote::quote! {
            impl<'r> ::axum_starter::Provider<'r, #provide_type> for #this{
                fn provide(&'r self) -> #provide_type{
                    # fetch
                }
            }
        };

        tokens.extend(token)
    }
}
