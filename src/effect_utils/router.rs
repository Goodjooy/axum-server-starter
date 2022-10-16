use std::convert::Infallible;

use axum::response::Response;
use hyper::{Body, Request};
use tower::Service;

use crate::PreparedEffect;

/// [PreparedEffect](crate::PreparedEffect) add route
///
/// ## Note
/// calling [Router::route](axum::Router::route)
pub struct Route<R>(&'static str, Option<R>);

impl<R> Route<R> {
    pub fn new(router: &'static str, service: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self(router, Some(service))
    }
}

impl<R> PreparedEffect for Route<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(&mut self, router: axum::Router) -> axum::Router {
        router.route(self.0, self.1.take().unwrap())
    }
}

/// [PreparedEffect](crate::PreparedEffect) merge router
///
/// ## Note
/// calling [Router::merge](axum::Router::merge)
pub struct Merge<R>(Option<R>);

impl<R> Merge<R> {
    pub fn new(merge: R) -> Self
    where
        axum::Router: From<R>,
    {
        Self(Some(merge))
    }
}
impl<R> PreparedEffect for Merge<R>
where
    axum::Router: From<R>,
{
    fn add_router(&mut self, router: axum::Router) -> axum::Router {
        router.merge(self.0.take().unwrap())
    }
}

/// [PreparedEffect](crate::PreparedEffect) nest router
///
/// ## Note
/// calling [Router::nest](axum::Router::nest)
pub struct Nest<R> {
    path: &'static str,
    router: Option<R>,
}

impl<R> Nest<R> {
    pub fn new(path: &'static str, router: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self {
            path,
            router: Some(router),
        }
    }
}

impl<R> PreparedEffect for Nest<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(&mut self, router: axum::Router) -> axum::Router {
        router.nest(self.path, self.router.take().unwrap())
    }
}

/// [PreparedEffect](crate::PreparedEffect) set fallback handle
///
/// ## Note
/// calling [Router::fallback](axum::Router::fallback)
pub struct Fallback<R> {
    service: Option<R>,
}

impl<R> PreparedEffect for Fallback<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    fn add_router(&mut self, router: axum::Router) -> axum::Router {
        router.fallback(self.service.take().unwrap())
    }
}

impl<R> Fallback<R> {
    pub fn new(service: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self {
            service: Some(service),
        }
    }
}
