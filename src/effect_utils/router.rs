use std::convert::Infallible;

use axum::response::Response;
use hyper::{Body, Request};
use tower::Service;

use crate::{PreparedEffect, RouteEffect};

/// [PreparedEffect](crate::PreparedEffect) add route
///
/// ## Note
/// calling [Router::route](axum::Router::route)
pub struct Route<R>(&'static str, R);

impl<R> Route<R> {
    pub fn new(router: &'static str, service: R) -> Self
    where
        R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
        R::Future: Send + 'static,
    {
        Self(router, service)
    }
}

impl<R> PreparedEffect for Route<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    type Extension = ();

    type Graceful = ();

    type Route = Self;

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), self, (), ())
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
    pub fn new(merge: R) -> Self
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
impl<R> PreparedEffect for Merge<R>
where
    axum::Router: From<R>,
{
    type Extension = ();

    type Graceful = ();

    type Route = Self;

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), self, (), ())
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
    pub fn new(path: &'static str, router: R) -> Self
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
impl<R> PreparedEffect for Nest<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    type Extension = ();

    type Graceful = ();

    type Route = Self;

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), self, (), ())
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
    pub fn new(service: R) -> Self
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

impl<R> PreparedEffect for Fallback<R>
where
    R: Service<Request<Body>, Response = Response, Error = Infallible> + Clone + Send + 'static,
    R::Future: Send + 'static,
{
    type Extension = ();

    type Graceful = ();

    type Route = Self;

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), self, (), ())
    }
}
