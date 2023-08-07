use heck::ToUpperCamelCase;
use syn::{
    spanned::Spanned, AngleBracketedGenericArguments, AssocType, Expr, Path, PathSegment, Type,
    TypeArray, TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

pub(crate) fn snake_to_upper(src: &str) -> String {
    ToUpperCamelCase::to_upper_camel_case(src)
}

macro_rules! darling_err {
    ($provider:expr) => {
        match $provider {
            Ok(v) => v,
            Err(err) => return err.write_errors().into(),
        }
    };
}

pub fn check_callable_expr(expr: &Expr) -> Result<(), syn::Error> {
    if let Expr::Path(_) | Expr::Closure(_) = expr {
        Ok(())
    } else {
        Err(syn::Error::new(expr.span(), "Expect `Path` or `Closure`"))
    }
}

pub(crate) fn check_accept_args_type(ty: &Type) -> Result<(), syn::Error> {
    match ty {
        Type::Array(TypeArray { elem, .. })
        | Type::Ptr(TypePtr { elem, .. })
        | Type::Reference(TypeReference { elem, .. })
        | Type::Slice(TypeSlice { elem, .. }) => check_accept_args_type(elem),

        Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) => {
            for PathSegment { arguments, .. } in segments {
                match arguments {
                    syn::PathArguments::None | syn::PathArguments::Parenthesized(_) => (),
                    syn::PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args,
                        ..
                    }) => {
                        for arg in args {
                            match arg {
                                syn::GenericArgument::AssocType(AssocType { ty, .. })
                                | syn::GenericArgument::Type(ty) => check_accept_args_type(ty)?,
                                _ => (),
                            }
                        }
                    }
                }
            }
            Ok(())
        }
        Type::Tuple(TypeTuple { elems, .. }) => {
            for elem in elems {
                check_accept_args_type(elem)?;
            }
            Ok(())
        }
        _ => Err(syn::Error::new(
            ty.span(),
            "`prepare` nonsupport this kind of function argument type",
        )),
    }
}

#[cfg(test)]
mod test {
    use crate::utils::snake_to_upper;

    #[test]
    fn test() {
        assert_eq!("Abc", snake_to_upper("abc"));
        assert_eq!("AbcCc", snake_to_upper("abc_cc"));
        assert_eq!("", snake_to_upper("_"));
        assert_eq!("AccBccDcc", snake_to_upper("acc_bcc_dcc"));
        assert_eq!("AccBccDcc", snake_to_upper("AccBccDcc"));
    }
}
