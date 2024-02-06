use std::{error, io};

use futures::future::LocalBoxFuture;
use futures::FutureExt;
use hyper_util::server::conn::auto::Builder;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// binding address provided by [ServeAddress]
pub trait BindServe: ServeAddress {
    fn bind(&self) -> LocalBoxFuture<io::Result<TcpListener>> {
        let addr = ServeAddress::get_address(self).into();
        TcpListener::bind(addr).boxed_local()
    }
}

/// get the address this server are going to bind with
pub trait ServeAddress {
    type Address: Into<SocketAddr>;
    fn get_address(&self) -> Self::Address;
}

impl<T: ServeAddress> BindServe for T {}

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
#[deprecated(since = "0.9.0", note = "axum current not support edit server")]
pub trait ConfigureServerEffect {
    fn effect_server<E>(&self, server: Builder<E>) -> Builder<E> {
        server
    }
}

pub use super::decorator::{EmptyDecorator, PrepareDecorator};
