use std::{any::Any, convert::Infallible, marker::PhantomData};

use axum::Extension;
use futures::future::{Ready, ok};
use tower::Layer;

use crate::{Prepare, PrepareMiddlewareEffect, StateCollector, TypeNotInState};

pub struct CarryToExtension<T>(PhantomData<T>);

impl<S, T: 'static + Clone + Send + Sync + Any> PrepareMiddlewareEffect<S> for CarryToExtension<T> {
    type Middleware = Extension<T>;

    fn take(self, states: &mut crate::StateCollector) -> Result<Self::Middleware, TypeNotInState> {
        let data = states.take_clone().unwrap();
        Ok(Extension(data))
    }
}

impl<T, C> Prepare<C> for CarryToExtension<T>
where
    T: 'static + Clone + Send + Sync + Any,
    C:'static
{
    type Effect = Self;

    type Error = Infallible;

    type Future = Ready<Result<Self, Infallible>>;

    fn prepare(self, _config: std::sync::Arc<C>) -> Self::Future {
        ok(self)
    }
}

impl<T> CarryToExtension<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub struct LazyInitial<F>(pub F);

impl<F, S, M> PrepareMiddlewareEffect<S> for LazyInitial<F>
where
    S: 'static,
    F: 'static + FnOnce(&mut StateCollector) -> Result<M, TypeNotInState>,
    M: Layer<S> + 'static,
{
    type Middleware = M;

    fn take(self, states: &mut crate::StateCollector) -> Result<Self::Middleware, TypeNotInState> {
        self.0(states)
    }
}

impl<T, S> PrepareMiddlewareEffect<S> for Extension<T>
where
    T: 'static + Clone + Send + Sync,
{
    type Middleware = Extension<T>;

    fn take(self, _: &mut StateCollector) -> Result<Self::Middleware, TypeNotInState> {
        Ok(self)
    }
}
