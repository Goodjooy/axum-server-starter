use quote::format_ident;
use syn::Type;

use syn::Ident;

use crate::utils::snake_to_upper;

use super::type_mapper::TypeMapper;

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

    #[darling(default, rename = "r#ref")]
    provide_ref: bool,

    #[darling(default)]
    ignore_global: bool,

    #[darling(default)]
    rename: Option<Ident>,

    #[darling(default, multiple)]
    map_to: Vec<TypeMapper>,
}

impl ProviderField {
    fn check_correct(self) -> darling::Result<Self> {
        let Self {
            ident,
            ty,
            transparent,
            mut skip,
            rename,
            map_to: aliases,
            provide_ref,
            ignore_global,
        } = self;

        match &ident {
            None => {
                Err(darling::Error::unexpected_type("Tuple Struct").with_span(&ident))?;
            }
            Some(ident) => {
                if *ident == "_" {
                    skip = true
                }
            }
        }

        if skip && (transparent || rename.is_some() || !aliases.is_empty() || provide_ref) {
            Err(darling::Error::duplicate_field("skip").with_span(&skip))?;
        }

        Ok(Self {
            ident,
            ty,
            transparent,
            skip,
            rename,
            map_to: aliases,
            provide_ref,
            ignore_global,
        })
    }

    pub fn into_info(self, outer_transparent: bool, outer_ref: bool) -> Option<FieldInfo> {
        let Self {
            ident,
            ty,
            transparent,
            skip,
            rename,
            map_to,
            provide_ref,
            ignore_global,
        } = self;
        let ident = ident?;
        let upper_ident = format_ident!("{}", snake_to_upper(&ident.to_string()));
        let transparent = transparent || (outer_transparent && !ignore_global);
        let provide_ref = provide_ref || (outer_ref && !ignore_global);
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
                mappers: map_to,
                provide_type: if !provide_ref {
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
    pub mappers: Vec<TypeMapper>,
}

#[derive(Debug, Clone, Copy)]
pub enum ProvideType {
    Ref,
    Owned,
}
