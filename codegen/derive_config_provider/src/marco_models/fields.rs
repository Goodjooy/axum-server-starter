use quote::format_ident;
use syn::Type;

use syn::Ident;

use crate::utils::snake_to_upper;

#[derive(Debug, darling::FromField)]
#[darling(attributes(provider), and_then = "ProviderField::check_correct")]
pub struct ProviderField {
    ident: Option<syn::Ident>,
    ty: Type,

    #[darling(default)]
    /// this make provide marco do not warp it
    transparent: bool,

    /// skip
    #[darling(default)]
    skip: bool,

    #[darling(default)]
    provide_owned: bool,

    #[darling(default)]
    rename: Option<Ident>,

    #[darling(default, multiple)]
    aliases: Vec<syn::Ident>,
}

impl ProviderField {
    fn check_correct(self) -> darling::Result<Self> {
        let Self {
            ident,
            ty,
            transparent,
            mut skip,
            rename,
            aliases,
            provide_owned,
        } = self;

        match &ident {
            None => {
                Err(darling::Error::unexpected_type("Tuple Struct").with_span(&ident))?;
            }
            Some(ident) => {
                if ident.to_string() == "_" {
                    skip = true
                }
            }
        }

        if skip && (transparent || rename.is_some() || !aliases.is_empty() || provide_owned) {
            Err(darling::Error::duplicate_field("skip").with_span(&skip))?;
        }

        Ok(Self {
            ident,
            ty,
            transparent,
            skip,
            rename,
            aliases,
            provide_owned,
        })
    }

    pub fn into_info(self) -> Option<FieldInfo> {
        let Self {
            ident,
            ty,
            transparent,
            skip,
            rename,
            aliases,
            provide_owned,
        } = self;
        let ident = ident?;
        let upper_ident = format_ident!("{}", snake_to_upper(&ident.to_string()));

        if skip {
            None
        } else {
            Some(FieldInfo {
                src_field_ident: ident,
                ty,

                wrapper_name: if !transparent {
                    Some(rename.unwrap_or(upper_ident))
                } else {
                    None
                },
                aliases,
                provide_type: if provide_owned {
                    ProvideType::Owned
                } else {
                    ProvideType::Ref
                },
            })
        }
    }
}

pub struct FieldInfo {
    pub src_field_ident: Ident,
    pub ty: Type,

    pub provide_type: ProvideType,
    pub wrapper_name: Option<Ident>,
    pub aliases: Vec<Ident>,
}
#[derive(Debug, Clone, Copy)]
pub enum ProvideType {
    Ref,
    Owned,
}
