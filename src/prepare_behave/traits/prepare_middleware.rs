use tower::{layer::util::Identity, Layer};

use crate::prepare_behave::effect_collectors::state_collector::StateCollector;

/// prepare for Middleware
///
/// it can adding middleware and state
pub trait PrepareMiddlewareEffect<S>: Sized + 'static {
    type Middleware: Layer<S> + 'static;

    fn take(self, states: &mut StateCollector) -> Self::Middleware;
}

impl<S> PrepareMiddlewareEffect<S> for () {
    type Middleware = Identity;

    fn take(self, _: &mut StateCollector) -> Self::Middleware {
        Identity::new()
    }
}
