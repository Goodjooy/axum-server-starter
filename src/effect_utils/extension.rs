use std::sync::Arc;

use crate::{EffectsCollector, ExtensionEffect};

/// [PreparedEffect](crate::PreparedEffect) adding extension
pub struct SetExtension<E>(E);

impl<E> SetExtension<Arc<E>>
where
    Arc<E>: Clone + Send + Sync + 'static,
{
    /// [PreparedEffect](crate::PreparedEffect) adding extension with [Arc](std::sync::Arc) wrapping
    pub fn arc(state: E) -> EffectsCollector<(), (), ((), SetExtension<Arc<E>>)> {
        SetExtension::<Arc<E>>::new(Arc::new(state))
    }

    pub fn arc_raw(state: E) -> Self {
        Self::new_raw(Arc::new(state))
    }
}

impl<E> SetExtension<E>
where
    E: Clone + Send + Sync + 'static,
{
    /// [PreparedEffect](crate::PreparedEffect) adding extension
    pub fn new(state: E) -> EffectsCollector<(), (), ((), SetExtension<E>)> {
        EffectsCollector::new().with_extension(Self::new_raw(state))
    }

    pub fn new_raw(state: E) -> Self {
        Self(state)
    }
}

impl<E> ExtensionEffect for SetExtension<E>
where
    E: Clone + Send + Sync + 'static,
{
    fn add_extension(self, extension: crate::ExtensionManage) -> crate::ExtensionManage {
        extension.add_extension(self.0)
    }
}
