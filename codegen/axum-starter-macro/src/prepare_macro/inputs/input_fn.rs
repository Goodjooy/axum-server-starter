use std::ops::Deref;

use syn::{
    punctuated::Punctuated, spanned::Spanned, ConstParam, FnArg, Generics, ItemFn, Lifetime,
    LifetimeDef, PatType, PredicateType, Token, Type, TypeParam, WherePredicate,
};

pub struct InputFn<'r> {
    pub is_async: bool,
    pub fn_name: &'r syn::Ident,
    pub generic: GenericWithBound<'r>,
    pub args_type: Vec<&'r syn::Type>,
    pub ret: Option<&'r Type>,
}

impl<'r> InputFn<'r> {
    pub fn from_fn_item(item: &'r ItemFn, lifetime: Option<&'r Lifetime>) -> syn::Result<Self> {
        let sig = &item.sig;
        if sig.constness.is_some() {
            Err(syn::Error::new(
                sig.constness.span(),
                "`prepare` not support const fn",
            ))?;
        }

        if sig.unsafety.is_some() {
            Err(syn::Error::new(
                sig.unsafety.span(),
                "`prepare` not support unsafe fn",
            ))?;
        }

        if sig.abi.is_some() {
            Err(syn::Error::new(
                sig.abi.span(),
                "`prepare` not support extern fn",
            ))?;
        }

        if let Some(FnArg::Receiver(r)) = sig.inputs.first() {
            Err(syn::Error::new(
                r.span(),
                "`prepare` not support fn with Self receiver",
            ))?;
        }

        let generic = GenericWithBound::new(&sig.generics, lifetime)?;
        let ret = match sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ref ty) => Some(ty.deref()),
        };
        Ok(Self {
            is_async: sig.asyncness.is_some(),
            fn_name: &sig.ident,
            args_type: sig
                .inputs
                .iter()
                .filter_map(|input| match input {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(PatType { ty, .. }) => Some(ty.as_ref()),
                })
                .collect(),
            generic,
            ret,
        })
    }
}

pub struct GenericWithBound<'r> {
    /// where bound
    pub where_closure: Option<Punctuated<PredicateType, Token![,]>>,
    /// type generic
    pub type_generic: Punctuated<&'r TypeParam, Token![,]>,
    pub const_generic: Punctuated<&'r ConstParam, Token![,]>,
}

impl<'r> GenericWithBound<'r> {
    fn new(generic: &'r Generics, lifetime: Option<&'r Lifetime>) -> syn::Result<Self> {
        // where bund only have type
        let where_bound = if let Some(bounds) = generic.where_clause.as_ref().map(|w| &w.predicates)
        {
            Some(
                bounds
                    .into_iter()
                    .map(|bound| match bound {
                        WherePredicate::Type(PredicateType {
                            lifetimes,
                            bounded_ty,
                            colon_token,
                            bounds,
                        }) => {
                            if lifetimes.is_some() {
                                Err(syn::Error::new(
                                    lifetimes.span(),
                                    "`prepare` not support bound with lifetime",
                                ))?;
                            };

                            let predicate_type = PredicateType {
                                lifetimes: None,
                                bounded_ty: bounded_ty.clone(),
                                colon_token: *colon_token,
                                bounds: bounds.clone(),
                            };

                            Ok(predicate_type)
                        }
                        other => Err(syn::Error::new(
                            other.span(),
                            "`prepare` only support generic bound for Type",
                        )),
                    })
                    .try_fold(Punctuated::<_, Token!(,)>::new(), |mut iter, token| {
                        token.map(|t| {
                            iter.push(t);
                            iter
                        })
                    })?,
            )
        } else {
            None
        };

        // if provide lifetime, there only on lifetime
        if let Some(lf) = lifetime {
            let mut lifetime_iter = generic.lifetimes();
            // if has at lest one lifetime, check is the same as provide
            match lifetime_iter.next() {
                Some(LifetimeDef { lifetime, .. }) if lifetime != lf => {
                    Err(syn::Error::new(
                        lifetime.span(),
                        "`prepare` only support lifetime equal to provide",
                    ))?;
                }
                _ => (),
            }
            // if more then on lifetime , error
            if let Some(ltd) = lifetime_iter.next() {
                Err(syn::Error::new(
                    ltd.span(),
                    "`prepare` only support signal lifetime",
                ))?;
            }
        } else {
            // if no provide , should no lifetime
            if let Some(ltd) = generic.lifetimes().next() {
                Err(syn::Error::new(
                    ltd.span(),
                    "`prepare` without provide lifetime not support lifetime",
                ))?;
            }
        }

        let this = Self {
            where_closure: where_bound,
            type_generic: generic.type_params().collect(),
            const_generic: generic.const_params().collect(),
        };

        Ok(this)
    }
}
