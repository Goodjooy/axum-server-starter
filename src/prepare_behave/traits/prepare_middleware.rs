use tower::Layer;

use crate::prepare_behave::effect_collectors::state_collector::StateCollector;

/// prepare for Middleware
///
/// it can adding middleware and state
pub trait MiddlewarePrepareEffect<S>: Sized + 'static {
    type Middleware: Layer<S> + 'static;

    fn take(self, states: &mut StateCollector) -> Self::Middleware;
}
