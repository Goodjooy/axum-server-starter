use std::marker::PhantomData;

use axum::{handler::Handler, routing::MethodRouter, Router};

use crate::prepare_behave::effect_traits::PrepareRouteEffect;

/// [PrepareRouteEffect] add route
///
/// ## Note
/// calling [Router::route](Router::route)
pub struct Route<S>(&'static str, MethodRouter<S>);

impl<S> Route<S> {
    pub fn new(router: &'static str, service: MethodRouter<S>) -> Self {
        Self(router, service)
    }
}

impl<S: 'static> PrepareRouteEffect<S> for Route<S> {
    fn set_route(self, route: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
    {
        route.route(self.0, self.1)
    }
}

/// [PrepareRouteEffect] merge router
///
/// ## Note
/// calling [Router::merge](Router::merge)
pub struct Merge<R>(R);

impl<R> Merge<R> {
    pub fn new(merge: R) -> Self
    where
        Router: From<R>,
    {
        Self(merge)
    }
}
impl<S, R> PrepareRouteEffect<S> for Merge<R>
where
    R: 'static,
    Router<S>: From<R>,
    S: Clone + Send + Sync + 'static,
{
    fn set_route(self, route: Router<S>) -> Router<S> {
        route.merge(self.0)
    }
}

/// [PrepareRouteEffect] nest router
///
/// ## Note
/// calling [Router::nest](Router::nest)
pub struct Nest<S> {
    path: &'static str,
    router: Router<S>,
}

impl<S> Nest<S> {
    pub fn new(path: &'static str, router: Router<S>) -> Self {
        Self { path, router }
    }
}

impl<S> PrepareRouteEffect<S> for Nest<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn set_route(self, route: Router<S>) -> Router<S> {
        route.nest(self.path, self.router)
    }
}

/// [PrepareRouteEffect] set fallback handle
///
/// ## Note
/// calling [Router::fallback](Router::fallback)
pub struct Fallback<H, T> {
    handle: H,
    __phantom: PhantomData<T>,
}

impl<R, T> Fallback<R, T> {
    pub fn new(handle: R) -> Self {
        Self {
            handle,
            __phantom: PhantomData,
        }
    }
}
impl<S, R, T> PrepareRouteEffect<S> for Fallback<R, T>
where
    R: Handler<T, S>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
{
    fn set_route(self, route: Router<S>) -> Router<S> {
        route.fallback(self.handle)
    }
}
