use axum::Router;
use http_body::Body;

/// prepare side effect of [PrepareRoute]
pub trait PrepareRouteEffect<B>: 'static + Sized
where
    B: Body + Send + 'static,
{
    fn set_route(self, route: Router<(), B>) -> Router<(), B> {
        route.route("/a", axum::routing::get(|| async { "aa" }))
    }
}
