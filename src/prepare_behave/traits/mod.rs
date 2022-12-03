pub mod prepare_middleware;
pub mod prepare_route;
pub mod prepare_state;

use std::{error::Error as StdError, future::IntoFuture, sync::Arc};

use futures::{future::Map, Future, FutureExt};

/// Prepare Task witch may return any kind of effect
pub trait Prepare<C: 'static> {
    /// the effect
    type Effect: 'static;
    /// the prepare can fall
    type Error: StdError + 'static;
    /// the future for preparing
    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;
    fn prepare(self, config: Arc<C>) -> Self::Future;
}

/// the prepare may fall
pub trait FalliblePrepare {
    /// the effect of prepare
    type Effect: 'static;
    /// the error when failure
    type Error: StdError + 'static;

    /// convent the Prepare result to Result
    fn to_result(self) -> Result<Self::Effect, Self::Error>;
}

impl<T: 'static, E: 'static + StdError> FalliblePrepare for Result<T, E> {
    type Effect = T;

    type Error = E;

    fn to_result(self) -> Result<Self::Effect, Self::Error> {
        self
    }
}

impl<F, C, Fut, Effect> Prepare<C> for F
where
    C: 'static,
    F: FnOnce(Arc<C>) -> Fut,
    Fut: Future<Output = Effect>,
    Effect: FalliblePrepare,
{
    type Effect = Effect::Effect;

    type Error = Effect::Error;

    type Future = Map<Fut, fn(Effect) -> Result<Self::Effect, Self::Error>>;

    fn prepare(self, config: Arc<C>) -> Self::Future {
        self(config).map(FalliblePrepare::to_result)
    }
}
