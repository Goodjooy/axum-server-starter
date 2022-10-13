use hyper::server::{conn::AddrIncoming, Builder};

use crate::PreparedEffect;

pub struct ConfigServer(Box<dyn Fn(Builder<AddrIncoming>) -> Builder<AddrIncoming>>);

impl PreparedEffect for ConfigServer {
    fn config_serve(
        &self,
        server: hyper::server::Builder<AddrIncoming>,
    ) -> hyper::server::Builder<AddrIncoming> {
        (self.0)(server)
    }
}

impl ConfigServer {
    pub fn new<F>(func: F) -> Self
    where
        F: Fn(Builder<AddrIncoming>) -> Builder<AddrIncoming> + 'static,
    {
        Self(Box::new(func))
    }
}
