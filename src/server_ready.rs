
use axum::{routing::IntoMakeService, Router};
use futures::Future;
use hyper::{server::conn::AddrIncoming, Body};

use crate::info;

/// all prepare task are done , the server is ready for launch
pub enum ServerReady<G> {
    Server(hyper::server::Server<AddrIncoming, IntoMakeService<Router<Body>>>),
    Graceful(G),
}

impl<G: Future<Output = hyper::Result<()>>> ServerReady<G> {
    /// start this server
    pub async fn launch(self) -> hyper::Result<()> {
        info!(service.status = "Starting");
        match self {
            ServerReady::Server(s) => s.await?,
            ServerReady::Graceful(g) => g.await?,
        };
        Ok(())
    }
}
