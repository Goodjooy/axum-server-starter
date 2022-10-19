use std::pin::Pin;

use futures::Future;

use crate::{EffectsCollector, GracefulEffect};

/// [PreparedEffect](crate::PreparedEffect) set graceful shutdown
pub struct SetGraceful(Option<Pin<Box<dyn Future<Output = ()>>>>);

impl SetGraceful {
    pub fn new<F>(future: F) -> EffectsCollector<(), ((), SetGraceful)>
    where
        F: Future<Output = ()> + 'static,
    {
        EffectsCollector::new().with_graceful(Self::new_raw(future))
    }
    pub fn new_raw<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self(Some(Box::pin(future) as Pin<Box<dyn Future<Output = ()>>>))
    }
}

impl GracefulEffect for SetGraceful {
    fn set_graceful(self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        self.0
    }
}
