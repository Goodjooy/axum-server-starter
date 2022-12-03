pub mod prepare_middleware;
pub mod prepare_route;
mod prepare_state;

use std::{error::Error as StdError, future::IntoFuture, sync::Arc};

pub trait Prepare {
    type Config;
    type Effect: FalliblePrepare;

    type Future: IntoFuture<Output = Self::Effect>;
    fn prepare(self, config: Arc<Self::Config>) -> Self::Future;
}

pub trait FalliblePrepare {
    type Error: StdError;
    type Effect;

    fn to_result(self) -> Result<Self::Effect, Self::Error>;
}

impl<T, E: StdError> FalliblePrepare for Result<T, E> {
    type Error = E;

    type Effect = T;

    fn to_result(self) -> Result<Self::Effect, Self::Error> {
        self
    }
}
