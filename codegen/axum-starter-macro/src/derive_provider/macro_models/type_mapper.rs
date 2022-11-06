use darling::FromMeta;
use syn::{spanned::Spanned, Expr, Lifetime, Type};

use crate::utils::check_callable_expr;

#[derive(Debug, FromMeta)]
#[darling(and_then = "TypeMapper::check")]
pub struct TypeMapper {
    pub ty: Type,
    pub by: Expr,
    pub lifetime: Option<String>,
    #[darling(skip, default)]
    pub lifetime_inner: Option<Lifetime>,
}

impl TypeMapper {
    pub fn check(self) -> Result<TypeMapper, darling::Error> {
        let inner = self
            .lifetime
            .as_deref()
            .map(syn::parse_str::<Lifetime>)
            .transpose()
            .map_err(darling::Error::from)
            .map_err(|err| err.with_span(&self.lifetime.span()))?;
        check_callable_expr(&self.by)?;
        Ok(TypeMapper {
            lifetime_inner: inner,
            ..self
        })
    }
}
