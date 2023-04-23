use std::pin::Pin;

use futures::Future;

use crate::{prepare_behave::EffectContainer, PrepareError, StateCollector};

pub(crate) mod concurrent_set;
pub(crate) mod serial_set;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

pub type ContainerResult<Route, Layer> = Result<EffectContainer<Route, Layer>, PrepareError>;

pub type ContainerFuture<R, L> = BoxFuture< ContainerResult<R, L>>;

pub type StateContainerResult = Result<StateCollector, PrepareError>;
pub type StateContainerFuture = BoxFuture< StateContainerResult>;
