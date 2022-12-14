use tower::{layer::util::Identity, Layer};

use crate::{prepare_behave::effect_collectors::state_collector::StateCollector, TypeNotInState};

/// prepare for Middleware
///
/// it can adding middleware and state
pub trait PrepareMiddlewareEffect<S>: Sized + 'static {
    type Middleware: Layer<S> + 'static;

    fn take(self, states: &mut StateCollector) -> Result<Self::Middleware, TypeNotInState>;
}

impl<S> PrepareMiddlewareEffect<S> for () {
    type Middleware = Identity;

    fn take(self, _: &mut StateCollector) -> Result<Self::Middleware, TypeNotInState> {
        Ok(Identity::new())
    }
}
