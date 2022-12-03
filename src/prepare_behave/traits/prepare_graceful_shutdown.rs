use std::{future::IntoFuture, sync::Arc};


/// prepare for Graceful shutdown
///
/// it can adding middleware and state
pub trait PrepareMiddleware {
    type Config;
    type Effect: IntoFuture<Output = ()>;

    type Error: super::StdError;

    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;

    fn prepare(self, config: Arc<Self::Config>) -> Self::Future;
}


