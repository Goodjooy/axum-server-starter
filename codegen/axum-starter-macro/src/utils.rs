use syn::{
    AngleBracketedGenericArguments, BareFnArg, GenericArgument, ParenthesizedGenericArguments,
    PathArguments, PathSegment, QSelf, ReturnType, Type, TypeArray, TypeBareFn, TypeGroup,
    TypePath, TypePtr, TypeReference, TypeSlice, TypeTuple,
};

pub(crate) fn snake_to_upper(src: &str) -> String {
    let mut st = String::with_capacity(src.len());
    let chars = src.chars();

    chars.fold((&mut st, true), |(st, need_upper), ch| {
        if ch == '_' {
            (st, true)
        } else {
            let ch = if need_upper {
                ch.to_ascii_uppercase()
            } else {
                ch
            };
            st.push(ch);
            (st, false)
        }
    });

    st
}

macro_rules! darling_err {
    ($provider:expr) => {
        match $provider {
            Ok(v) => v,
            Err(err) => return err.write_errors().into(),
        }
    };
}

pub fn verify_can_bound(ty: &Type) -> bool {
    match ty {
        Type::Ptr(TypePtr { elem, .. })
        | Type::Array(TypeArray { elem, .. })
        | Type::Reference(TypeReference { elem, .. })
        | Type::Slice(TypeSlice { elem, .. })
        | Type::Group(TypeGroup { elem, .. }) => verify_can_bound(elem),
        Type::BareFn(TypeBareFn { inputs, output, .. }) => {
            inputs
                .iter()
                .all(|BareFnArg { ty, .. }| verify_can_bound(ty))
                && match output {
                    ReturnType::Default => true,
                    ReturnType::Type(_, ty) => verify_can_bound(ty),
                }
        }
        Type::Path(TypePath {
            qself,
            path: syn::Path { segments, .. },
        }) => {
            let v = match qself {
                Some(QSelf { ty, .. }) => verify_can_bound(ty),
                None => true,
            } && segments
                .iter()
                .all(|PathSegment { arguments, .. }| match arguments {
                    PathArguments::None => true,
                    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        args, ..
                    }) => args.iter().all(|generic| match generic {
                        GenericArgument::Type(ty) => verify_can_bound(ty),
                        _ => true,
                    }),
                    PathArguments::Parenthesized(ParenthesizedGenericArguments {
                        inputs,
                        output,
                        ..
                    }) => {
                        inputs.iter().all(verify_can_bound)
                            && match output {
                                ReturnType::Default => true,
                                ReturnType::Type(_, ty) => verify_can_bound(ty),
                            }
                    }
                });
            v
        }
        Type::Never(_) | Type::TraitObject(_) => true,
        Type::Tuple(TypeTuple { elems, .. }) => elems.iter().all(verify_can_bound),
        _ => false,
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
