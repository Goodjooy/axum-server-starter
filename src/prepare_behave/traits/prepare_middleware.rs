use std::{future::IntoFuture, sync::Arc};

use tower::Layer;

/// prepare for Middleware
///
/// it can adding middleware and state
pub trait PrepareMiddleware {
    type Config;
    type Effect: MiddlewarePrepareEffect;

    type Error: super::StdError;

    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;

    fn prepare(self, config: Arc<Self::Config>) -> Self::Future;
}

pub trait MiddlewarePrepareEffect: Sized + 'static {
    type Middleware<S>: Layer<S> + 'static;

    type StateType: 'static;

    fn take<S>(self) -> (Self::Middleware<S>, Self::StateType);
}
