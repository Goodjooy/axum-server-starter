use std::marker::PhantomData;

use axum::{handler::Handler, routing::MethodRouter, Router};

use crate::prepare_behave::effect_traits::PrepareRouteEffect;

/// [PrepareRouteEffect] add route
///
/// ## Note
/// calling [Router::route](axum::Router::route)
pub struct Route<S, B>(&'static str, MethodRouter<S, B>);

impl<S, B> Route<S, B> {
    pub fn new(router: &'static str, service: MethodRouter<S, B>) -> Self {
        Self(router, service)
    }
}

impl<S: 'static, B: 'static> PrepareRouteEffect<S, B> for Route<S, B> {
    fn set_route(self, route: axum::Router<S, B>) -> axum::Router<S, B>
    where
        S: Clone + Send + Sync + 'static,
        B: http_body::Body + Send + 'static,
    {
        route.route(self.0, self.1)
    }
}

/// [PrepareRouteEffect] merge router
///
/// ## Note
/// calling [Router::merge](axum::Router::merge)
pub struct Merge<R>(R);

impl<R> Merge<R> {
    pub fn new(merge: R) -> Self
    where
        axum::Router: From<R>,
    {
        Self(merge)
    }
}
impl<S, B, R> PrepareRouteEffect<S, B> for Merge<R>
where
    R: 'static,
    axum::Router<S, B>: From<R>,
    S: Clone + Send + Sync + 'static,
    B: http_body::Body + Send + 'static,
{
    fn set_route(self, route: axum::Router<S, B>) -> axum::Router<S, B> {
        route.merge(self.0)
    }
}

/// [PrepareRouteEffect] nest router
///
/// ## Note
/// calling [Router::nest](axum::Router::nest)
pub struct Nest<S, B> {
    path: &'static str,
    router: Router<S, B>,
}

impl<S, B> Nest<S, B> {
    pub fn new(path: &'static str, router: Router<S, B>) -> Self {
        Self { path, router }
    }
}

impl<S, B> PrepareRouteEffect<S, B> for Nest<S, B>
where
    S: Clone + Send + Sync + 'static,
    B: http_body::Body + Send + 'static,
{
    fn set_route(self, route: axum::Router<S, B>) -> axum::Router<S, B> {
        route.nest(self.path, self.router)
    }
}

/// [PrepareRouteEffect] set fallback handle
///
/// ## Note
/// calling [Router::fallback](axum::Router::fallback)
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
impl<S, B, R, T> PrepareRouteEffect<S, B> for Fallback<R, T>
where
    R: Handler<T, S, B>,
    T: 'static,
    S: Clone + Send + Sync + 'static,
    B: http_body::Body + Send + 'static,
{
    fn set_route(self, route: axum::Router<S, B>) -> axum::Router<S, B> {
        route.fallback(self.handle)
    }
}
