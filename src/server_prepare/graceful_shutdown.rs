use futures::{future::Ready, Future};

use crate::ServerPrepare;

pub struct NoGraceful;
pub struct Graceful<Fut>(Fut);

impl<Fut> FetchGraceful for Graceful<Fut>
where
    Fut: Future<Output = ()>,
{
    type Future = Fut;

    fn get_graceful(self) -> Option<Self::Future> {
        Some(self.0)
    }
}

pub trait FetchGraceful {
    type Future: Future<Output = ()>;
    fn get_graceful(self) -> Option<Self::Future>;
}

impl FetchGraceful for NoGraceful {
    type Future = Ready<()>;

    fn get_graceful(self) -> Option<Self::Future> {
        None
    }
}

impl<C, FutEffect, Log, State> ServerPrepare<C, FutEffect, Log, State, NoGraceful> {
    pub fn graceful_shutdown<Fut>(
        self,
        future: Fut,
    ) -> ServerPrepare<C, FutEffect, Log, State, Graceful<Fut>>
    where
        Fut: Future<Output = ()>,
    {
        ServerPrepare::new(self.prepares, Graceful(future), self.span)
    }
}
