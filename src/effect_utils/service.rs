use hyper::server::{conn::AddrIncoming, Builder};

use crate::{PreparedEffect, ServerEffect};

/// [PreparedEffect](crate::PreparedEffect) configure [Builder](hyper::server::Builder)
pub struct ConfigServer(Box<dyn FnOnce(Builder<AddrIncoming>) -> Builder<AddrIncoming>>);

impl ServerEffect for ConfigServer {
    fn config_serve(self, server: Builder<AddrIncoming>) -> Builder<AddrIncoming> {
        (self.0)(server)
    }
}

impl PreparedEffect for ConfigServer {
    type Extension = ();

    type Graceful = ();

    type Route = ();

    type Server = Self;

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), (), (), self)
    }
}

impl ConfigServer {
    /// using a function to configure [Builder](hyper::server::Builder)
    pub fn new<F>(func: F) -> Self
    where
        F: FnOnce(Builder<AddrIncoming>) -> Builder<AddrIncoming> + 'static,
    {
        Self(Box::new(func))
    }
}
