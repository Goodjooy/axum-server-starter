use std::{future::IntoFuture, sync::Arc};

use axum::Router;

/// prepare for [Route](axum::Router)
///
/// it can adding route and State
pub trait PrepareRoute<B>
where
    B: http_body::Body + Send + 'static,
{
    type Config;
    type Effect: PrepareRouteEffect<B>;

    type Error: super::StdError;

    type Future: IntoFuture<Output = Result<Self::Effect, Self::Error>>;

    fn prepare(self, config: Arc<Self::Config>) -> Self::Future;
}

/// prepare side effect of [PrepareRoute]
pub trait PrepareRouteEffect<B>: 'static + Sized
where
    B: http_body::Body + Send + 'static,
{
    fn set_route(self, route: Router<(), B>) -> Router<(), B> {
        route.route("/a", axum::routing::get(|| async { "aa" }))
    }
}
