use axum::Router;

/// [Prepare] effect may apply [Router::route], [Router::nest], [Router::fallback], [Router::merge] on the root [Router]
pub trait RouteEffect: Sized {
    /// prepare want to adding routing on the root router
    ///
    /// ## Note
    /// [ExtensionEffect::add_extension] will be applied after [RouteEffect::add_router] being applied.
    ///
    /// the router adding by a [RouteEffect] can safely using Extension adding by
    /// [ExtensionEffect::add_extension] in the same [PrepareEffect](crate::PreparedEffect)
    fn add_router<S>(self, router: Router<S>) -> Router<S> 
    where
    S: Send + Sync + 'static + Clone,
    {
        router
    }
}