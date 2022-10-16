use std::pin::Pin;

use futures::Future;

use crate::{GracefulEffect, PreparedEffect};

/// [PreparedEffect](crate::PreparedEffect) set graceful shutdown
pub struct SetGraceful(Option<Pin<Box<dyn Future<Output = ()>>>>);

impl SetGraceful {
    pub fn new<F>(future: F) -> Self
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

impl PreparedEffect for SetGraceful {
    type Extension = ();

    type Graceful = Self;

    type Route = ();

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), (), self, ())
    }
}
