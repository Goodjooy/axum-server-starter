use std::error;

use hyper::server::conn::AddrIncoming;

/// get the address this server are going to bind with
pub trait ServeAddress {
    type Address: Into<std::net::SocketAddr>;
    fn get_address(&self) -> Self::Address;
}

/// init the logger of this server by the Config
///
/// init logger require **sync**
pub trait LoggerInitialization {
    type Error: error::Error;
    fn init_logger(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// change the server configure
pub trait ConfigureServerEffect {
    fn effect_server(
        &self,
        server: hyper::server::Builder<AddrIncoming>,
    ) -> hyper::server::Builder<AddrIncoming> {
        server
    }
}
