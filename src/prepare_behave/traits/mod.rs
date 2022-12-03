pub mod prepare_middleware;
pub mod prepare_route;
pub mod prepare_state;

use std::{error::Error as StdError, future::IntoFuture, sync::Arc};

pub trait Prepare<C: 'static> {
    type Effect: 'static;
    type Error: StdError + 'static;

    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;
    fn prepare(self, config: Arc<C>) -> Self::Future;
}
