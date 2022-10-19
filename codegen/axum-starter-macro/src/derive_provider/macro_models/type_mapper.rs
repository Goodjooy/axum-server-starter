use darling::FromMeta;
use syn::{Lifetime, Path, Type, spanned::Spanned};

#[derive(Debug, FromMeta)]
#[darling(and_then = "TypeMapper::check")]
pub struct TypeMapper {
    pub ty: Type,
    pub by: Path,
    pub lifetime: Option<String>,
    #[darling(skip, default)]
    pub lifetime_inner: Option<Lifetime>,
}

impl TypeMapper {
    pub fn check(self) -> Result<TypeMapper, darling::Error> {
        let inner = self
            .lifetime
            .as_ref()
            .map(String::as_str)
            .map(syn::parse_str::<Lifetime>)
            .transpose()
            .map_err(darling::Error::from)
            .map_err(|err|err.with_span(&self.lifetime.span()))
            ?;
        Ok(TypeMapper {
            lifetime_inner: inner,
            ..self
        })
    }
}