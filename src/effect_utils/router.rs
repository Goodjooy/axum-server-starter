use std::convert::Infallible;

use axum::response::Response;
use hyper::{Body, Request};
use tower::Service;

use crate::{EffectsCollector, RouteEffect};

/// [PreparedEffect](crate::PreparedEffect) add route
///
/// ## Note
/// calling [Router::route](axum::Router::route)
pub struct Route<R>(&'static str, R);

impl<R> Route<R> {
    pub fn new(router: &'static str, service: R) -> EffectsCollector<((), Route<R>)>
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        EffectsCollector::new().with_route(Self::new_raw(router, service))
    }

    pub fn new_raw(router: &'static str, service: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self(router, service)
    }
}

impl<R> RouteEffect for Route<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(self, router: axum::Router) -> axum::Router {
        router.route(self.0, self.1)
    }
}

/// [PreparedEffect](crate::PreparedEffect) merge router
///
/// ## Note
/// calling [Router::merge](axum::Router::merge)
pub struct Merge<R>(R);

impl<R> Merge<R> {
    pub fn new(merge: R) -> EffectsCollector<((), Merge<R>)>
    where
        axum::Router: From<R>,
    {
        EffectsCollector::new().with_route(Self::new_raw(merge))
    }
    pub fn new_raw(merge: R) -> Self
    where
        axum::Router: From<R>,
    {
        Self(merge)
    }
}
impl<R> RouteEffect for Merge<R>
where
    axum::Router: From<R>,
{
    fn add_router(self, router: axum::Router) -> axum::Router {
        router.merge(self.0)
    }
}

/// [PreparedEffect](crate::PreparedEffect) nest router
///
/// ## Note
/// calling [Router::nest](axum::Router::nest)
pub struct Nest<R> {
    path: &'static str,
    router: R,
}

impl<R> Nest<R> {
    pub fn new(path: &'static str, router: R) -> EffectsCollector<((), Nest<R>)>
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        EffectsCollector::new().with_route(Self::new_raw(path, router))
    }
    pub fn new_raw(path: &'static str, router: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self { path, router }
    }
}

impl<R> RouteEffect for Nest<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(self, router: axum::Router) -> axum::Router {
        router.nest(self.path, self.router)
    }
}

/// [PreparedEffect](crate::PreparedEffect) set fallback handle
///
/// ## Note
/// calling [Router::fallback](axum::Router::fallback)
pub struct Fallback<R> {
    service: R,
}

impl<R> Fallback<R> {
    pub fn new(service: R) -> EffectsCollector<((), Fallback<R>)>
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        EffectsCollector::new().with_route(Self::new_raw(service))
    }
    pub fn new_raw(service: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self { service }
    }
}
impl<R> RouteEffect for Fallback<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(self, router: axum::Router) -> axum::Router {
        router.fallback(self.service)
    }
}
