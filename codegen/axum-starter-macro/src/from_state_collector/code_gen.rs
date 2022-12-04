use darling::ToTokens;
use proc_macro2::Ident;
use syn::{punctuated::Punctuated, Token, Type};

pub struct GenImplFromState<'s> {
    pub ident: &'s Ident,
    pub fields: Vec<(Option<Ident>, Type)>,
}

impl<'s> ToTokens for GenImplFromState<'s> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let GenImplFromState { ident, fields } = self;

        let field_gen = fields
            .iter()
            .map(|(field, ty)| {
                if let Some(field) = field {
                    quote::quote!(#field : collector.take::<#ty>()? )
                } else {
                    quote::quote!(collector.take::<#ty>()?)
                }
            })
            .collect::<Punctuated<_, Token!(,)>>();

        let self_construct = if fields.is_empty() {
            quote::quote!(#ident)
        } else if fields.iter().all(|(f, _)| f.is_none()) {
            quote::quote!(
                #ident (
                    #field_gen
                )
            )
        } else {
            quote::quote!(
                # ident {
                    # field_gen
                }
            )
        };

        let token = quote::quote! {
            impl ::axum_starter::FromStateCollector for #ident {
                fn fetch_mut(
                    collector: &mut ::axum_starter::StateCollector,
                ) -> core::result::Result<Self, ::axum_starter::TypeNotInState> {
                    core::result::Result::Ok(
                        #self_construct
                    )
                }
            }
        };

        tokens.extend(token)
    }
}
