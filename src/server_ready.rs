use futures::Future;

/// all prepare task are done , the server is ready for launch
pub enum ServerReady<G, S> {
    Server(S),
    Graceful(G),
}

impl<S: Future<Output = hyper::Result<()>>, G: Future<Output = hyper::Result<()>>>
    ServerReady<S, G>
{
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
