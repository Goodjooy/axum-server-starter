use darling::util::Override;
use syn::{Path, Type};

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(conf), supports(struct_named))]
pub struct DeriveInput {
    pub(super) address: Address,
    #[darling(default)]
    pub(super) logger: Option<Logger>,
    #[darling(default)]
    pub(super) server: Override<Path>,
    pub(super) ident: syn::Ident,
}

impl DeriveInput {}

#[derive(Debug, darling::FromMeta)]
pub enum Address {
    Provide(Override<Provider>),
    Func {
        path: Path,
        #[darling(default)]
        ty: Option<Type>,
    },
}

#[derive(Debug, darling::FromMeta)]
pub struct Provider {
    pub(super) ty: Type,
}

#[derive(Debug, darling::FromMeta)]
pub struct Logger {
    pub(super) func: Path,
    pub(super) error: Type,
}
