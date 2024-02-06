use std::ops::Deref;

use crate::prepare_macro::DEFAULT_LIFETIME_SYMBOL;
use syn::visit_mut::VisitMut;
use syn::{
    punctuated::Punctuated, spanned::Spanned, visit_mut, ConstParam, FnArg, Generics, ItemFn,
    Lifetime, LifetimeParam, Pat, PatType, PredicateType, Stmt, Token, Type, TypeParam,
    TypeReference, WherePredicate,
};

pub struct ArgInfo {
    pub patten: Box<Pat>,
    pub ty: Box<Type>,
}

impl ArgInfo {
    fn new(pat: PatType) -> Self {
        Self {
            patten: pat.pat,
            ty: pat.ty,
        }
    }
}

pub struct InputFn<'r> {
    pub generic: GenericWithBound<'r>,
    pub args_type: Vec<ArgInfo>,
    pub ret: Option<&'r Type>,
    pub fn_body: &'r [Stmt],
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
        let lt_symbol = lifetime
            .as_ref()
            .map(|lt| lt.to_string())
            .unwrap_or_else(|| String::from(DEFAULT_LIFETIME_SYMBOL));
        let mut lifetime_visitor = LifetimeVisitor::new(&lt_symbol);
        let mut fn_args = sig.inputs.clone();
        for arg_types in fn_args.iter_mut() {
            lifetime_visitor.visit_fn_arg_mut(arg_types);
        }
        if let Some(err) = lifetime_visitor.failure {
            Err(syn::Error::new(
                err.span(),
                format!("unknown lifetime[{err}]"),
            ))?;
        }

        let generic = GenericWithBound::new(&sig.generics, lifetime)?;
        let ret = match sig.output {
            syn::ReturnType::Default => None,
            syn::ReturnType::Type(_, ref ty) => Some(ty.deref()),
        };
        Ok(Self {
            args_type: fn_args
                .into_iter()
                .filter_map(|input| match input {
                    FnArg::Receiver(_) => None,
                    FnArg::Typed(pat_type) => Some(ArgInfo::new(pat_type)),
                })
                .collect(),
            generic,
            ret,
            fn_body: item.block.stmts.as_slice(),
        })
    }
}

pub struct GenericWithBound<'r> {
    /// where bound
    pub where_closure: Option<Punctuated<PredicateType, Token![,]>>,
    /// type generic
    pub type_generic: Punctuated<&'r TypeParam, Token![,]>,
    pub const_generic: Punctuated<&'r ConstParam, Token![,]>,
    pub origin: &'r Generics,
}

impl<'r> GenericWithBound<'r> {
    fn new(generic: &'r Generics, lifetime: Option<&'r Lifetime>) -> syn::Result<Self> {
        // where bound only have type
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
                Some(LifetimeParam { lifetime, .. }) if lifetime != lf => {
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
            origin: generic,
            where_closure: where_bound,
            type_generic: generic.type_params().collect(),
            const_generic: generic.const_params().collect(),
        };

        Ok(this)
    }
}

struct LifetimeVisitor<'r> {
    symbol: &'r str,
    failure: Option<Lifetime>,
}

impl<'r> LifetimeVisitor<'r> {
    fn new(symbol: &'r str) -> Self {
        Self {
            symbol,
            failure: None,
        }
    }
}

impl VisitMut for LifetimeVisitor<'_> {
    fn visit_lifetime_mut(&mut self, i: &mut Lifetime) {
        'lt: {
            if self.failure.is_some() {
                break 'lt;
            }
            let span = i.span();
            let lifetime = if i.ident == self.symbol[1..] || i.ident == "static" {
                i.clone()
            } else if i.ident == "_" {
                Lifetime::new(self.symbol, span)
            } else {
                self.failure = i.clone().into();
                break 'lt;
            };

            *i = lifetime;
        }
        visit_mut::visit_lifetime_mut(self, i);
    }

    fn visit_type_reference_mut(&mut self, i: &mut TypeReference) {
        'lt: {
            if self.failure.is_some() {
                break 'lt;
            }
            let lifetime = match i.lifetime.clone() {
                None => Lifetime::new(self.symbol, i.lifetime.span()),
                Some(lifetime) if lifetime.ident == "_" => {
                    Lifetime::new(self.symbol, lifetime.span())
                }
                Some(lifetime)
                    if lifetime.ident == self.symbol[1..] || lifetime.ident == "static" =>
                {
                    lifetime
                }
                lifetime => {
                    self.failure = lifetime;
                    break 'lt;
                }
            };
            let _ = i.lifetime.insert(lifetime);
        };
        visit_mut::visit_type_reference_mut(self, i);
    }
}
