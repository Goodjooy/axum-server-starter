use std::convert::Infallible;

use axum::Extension;
use futures::future::{ok, Ready};

use crate::Prepare;

/// help types for apply effect on Middleware
pub mod middleware;
/// help types for apply effect on State
pub mod state;

/// help types for apply effect on [Router](axum::Router)
pub mod router;

impl<C: 'static, T: 'static> Prepare<C> for Extension<T> {
    type Effect = Self;

    type Error = Infallible;

    type Future = Ready<Result<Self, Infallible>>;

    fn prepare(self, _config: std::sync::Arc<C>) -> Self::Future {
        ok(self)
    }
}
