use std::sync::Arc;

use crate::PreparedEffect;

/// [PreparedEffect](crate::PreparedEffect) adding extension
pub struct SetExtension<E>(E);

impl<E> SetExtension<Arc<E>>
where
    Arc<E>: Clone + Send + Sync + 'static,
{   
    /// [PreparedEffect](crate::PreparedEffect) adding extension with [Arc](std::sync::Arc) wrapping
    pub fn arc(state: E) -> Self {
        Self(Arc::new(state))
    }
}

impl<E> SetExtension<E>
where
    E: Clone + Send + Sync + 'static,
{
    /// [PreparedEffect](crate::PreparedEffect) adding extension
    pub fn new(state: E) -> Self {
        Self(state)
    }
}

impl<E> PreparedEffect for SetExtension<E>
where
    E: Clone + Send + Sync + 'static,
{
    fn add_extension(&mut self, extension: crate::ExtensionManage) -> crate::ExtensionManage {
        extension.add_extension(self.0.clone())
    }
}
