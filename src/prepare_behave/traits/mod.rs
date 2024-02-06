pub mod prepare_middleware;
pub mod prepare_route;
pub mod prepare_state;

use std::{error::Error as StdError, future::IntoFuture, sync::Arc};

use futures::Future;

/// Prepare Task witch may return any kind of effect
pub trait Prepare<C: 'static>: 'static {
    /// the effect
    type Effect: 'static;
    /// prepare error
    type Error: StdError + 'static;
    /// the future for preparing
    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>> + 'static;
    fn prepare(self, config: Arc<C>) -> Self::Future;
}

impl<F, C, Fut, Effect, Error> Prepare<C> for F
where
    C: 'static,
    F: FnOnce(Arc<C>) -> Fut + 'static,
    Fut: Future<Output = Result<Effect, Error>> + 'static,
    Effect: 'static,
    Error: 'static + StdError,
{
    type Effect = Effect;

    type Error = Error;

    type Future = Fut;

    fn prepare(self, config: Arc<C>) -> Self::Future {
        self(config)
    }
}
