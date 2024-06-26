use futures::{future::Ready, Future};

use crate::ServerPrepare;

pub struct NoGraceful;
pub struct Graceful<Fut>(Fut);

impl<Fut> FetchGraceful for Graceful<Fut>
where
    Fut: Future<Output = ()> + Send + 'static,
{
    type Future = Fut;

    fn get_graceful(self) -> Option<Self::Future> {
        Some(self.0)
    }
}

pub trait FetchGraceful {
    type Future: Future<Output = ()> + Send + 'static;
    fn get_graceful(self) -> Option<Self::Future>;
}

impl FetchGraceful for NoGraceful {
    type Future = Ready<()>;

    fn get_graceful(self) -> Option<Self::Future> {
        None
    }
}

impl<C, FutEffect, Log, State, Decorator>
    ServerPrepare<C, FutEffect, Log, State, NoGraceful, Decorator>
{
    /// set the graceful shutdown signal
    pub fn graceful_shutdown<Fut>(
        self,
        future: Fut,
    ) -> ServerPrepare<C, FutEffect, Log, State, Graceful<Fut>, Decorator>
    where
        Fut: Future<Output = ()>,
    {
        ServerPrepare::new(self.prepares, Graceful(future), self.state, self.span)
    }
}
