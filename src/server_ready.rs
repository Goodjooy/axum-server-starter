use std::future::IntoFuture;

use axum::{routing::IntoMakeService, Router};
use hyper::{server::conn::AddrIncoming, Body};

pub enum ServerReady<I, M, G> {
    Server(hyper::server::Server<I, M>),
    Graceful(G),
}

impl<G: IntoFuture<Output = hyper::Result<()>>>
    ServerReady<AddrIncoming, IntoMakeService<Router<Body>>, G>
{
    pub async fn launch(self) -> hyper::Result<()> {
        match self {
            ServerReady::Server(s) => s.await?,
            ServerReady::Graceful(g) => g.await?,
        };
        Ok(())
    }
}
