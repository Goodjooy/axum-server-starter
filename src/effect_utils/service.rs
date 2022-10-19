use hyper::server::{conn::AddrIncoming, Builder};

use crate::{EffectsCollect, ServerEffect};

/// [PreparedEffect](crate::PreparedEffect) configure [Builder](hyper::server::Builder)
pub struct ConfigServer(Box<dyn FnOnce(Builder<AddrIncoming>) -> Builder<AddrIncoming>>);

impl ServerEffect for ConfigServer {
    fn config_serve(self, server: Builder<AddrIncoming>) -> Builder<AddrIncoming> {
        (self.0)(server)
    }
}

impl ConfigServer {
    /// using a function to configure [Builder](hyper::server::Builder)
    pub fn new<F>(func: F) -> EffectsCollect<(), (), (), ((), ConfigServer)>
    where
        F: FnOnce(Builder<AddrIncoming>) -> Builder<AddrIncoming> + 'static,
    {
        EffectsCollect::new().with_server(Self::new_raw(func))
    }
    /// using a function to configure [Builder](hyper::server::Builder)
    pub fn new_raw<F>(func: F) -> Self
    where
        F: FnOnce(Builder<AddrIncoming>) -> Builder<AddrIncoming> + 'static,
    {
        Self(Box::new(func))
    }
}
