use std::io;
use futures::Future;

/// all prepare task are done , the server is ready for launch
pub enum ServerReady<G, S> {
    Server(S),
    Graceful(G),
}

impl<S: Future<Output = io::Result<()>>, G: Future<Output = io::Result<()>>>
    ServerReady<S, G>
{
    /// start this server
    pub async fn launch(self) -> io::Result<()> {
        info!(service.status = "Starting");
        match self {
            ServerReady::Server(s) => s.await?,
            ServerReady::Graceful(g) => g.await?,
        };
        Ok(())
    }
}
