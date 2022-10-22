use std::future::IntoFuture;

use axum::{routing::IntoMakeService, Router};
use hyper::{server::conn::AddrIncoming, Body};

use crate::info;

/// all prepare task are done , the server is ready for launch
pub enum ServerReady<G> {
    Server(hyper::server::Server<AddrIncoming, IntoMakeService<Router<Body>>>),
    Graceful(G),
}

impl<G: IntoFuture<Output = hyper::Result<()>>> ServerReady<G> {
    /// start this server
    pub async fn launch(self) -> hyper::Result<()> {
        info!("Starting server");
        match self {
            ServerReady::Server(s) => s.await?,
            ServerReady::Graceful(g) => g.await?,
        };
        Ok(())
    }
}
