use std::{future::IntoFuture, sync::Arc};

/// Prepare for Global State
///
/// for instance the Connection Pool of Database
pub trait PrepareState: Sized {
    type Config;
    type Effect: PrepareStateEffect;

    type Error: super::StdError;

    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;

    fn prepare(self, config: Arc<Self::Config>) -> Self::Future;
}

/// prepare side effect of [PrepareState]
pub trait PrepareStateEffect:'static {
    type StateType: 'static;

    fn take_state(self) -> Self::StateType;
}
