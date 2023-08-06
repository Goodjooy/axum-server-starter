use std::error;
use std::fmt::Display;
use std::net::SocketAddr;

use hyper::server::accept::Accept;
use hyper::server::conn::AddrIncoming;
use hyper::server::Builder;
use hyper::Server;

/// binding to any kind of income stream
///
/// ## Using Cases
/// 1. Accept Data Stream from other source.
/// For example from UDS, Pipe or even Files
/// 2. Adding extra functional on exist [`Accept`] type.
/// For example adding logging request ip address on [`AddrIncoming`]
pub trait BindServe {
    /// the Accept type
    type A: Accept;
    /// the listen source for logger
    type Target: Display;

    /// get where the binder listen for
    fn listen_target(&self) -> Self::Target;

    /// create listener, ready for listen incoming streams
    fn create_listener(&self) -> Self::A;

    /// bind to listen target
    fn bind(&self) -> Builder<Self::A> {
        Server::builder(self.create_listener())
    }
}

/// get the address this server are going to bind with
pub trait ServeAddress {
    type Address: Into<SocketAddr>;
    fn get_address(&self) -> Self::Address;
}

impl<T> BindServe for T
where
    T: ServeAddress,
{
    type A = AddrIncoming;
    type Target = SocketAddr;

    fn listen_target(&self) -> Self::Target {
        self.get_address().into()
    }

    fn create_listener(&self) -> Self::A {
        let addr = &self.get_address().into();
        AddrIncoming::bind(addr).unwrap_or_else(|e| panic!("error bind to {addr} {e}"))
    }
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
pub trait ConfigureServerEffect<A = AddrIncoming>
where
    A: Accept,
{
    fn effect_server(&self, server: Builder<A>) -> Builder<A> {
        server
    }
}

pub use super::decorator::{EmptyDecorator, PrepareDecorator};
