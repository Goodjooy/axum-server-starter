use darling::util::Override;
use syn::{Expr, Path, Type};

use crate::utils::check_callable_expr;

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(conf), supports(struct_named))]
pub struct DeriveInput {
    #[darling(default)]
    pub(super) address: Option<Address>,
    #[darling(default)]
    pub(super) logger: Option<Logger>,
    #[darling(default)]
    pub(super) server: Override<Path>,
    pub(super) ident: syn::Ident,
}

#[derive(Debug, darling::FromMeta)]
pub enum Address {
    Provide(Override<Provider>),
    Func {
        path: Expr,
        #[darling(default)]
        ty: Option<Type>,
        #[darling(default)]
        associate: bool,
    },
}

#[derive(Debug, darling::FromMeta)]
pub struct Provider {
    pub(super) ty: Type,
}

#[derive(Debug, darling::FromMeta)]
#[darling(and_then = "Self::check_expr")]
pub struct Logger {
    pub(super) func: Expr,
    pub(super) error: Type,
    #[darling(default)]
    pub(super) associate: bool,
}

impl Logger {
    fn check_expr(self) -> darling::Result<Self> {
        check_callable_expr(&self.func)?;
        Ok(self)
    }
}
