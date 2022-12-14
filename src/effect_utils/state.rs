use std::sync::Arc;

use tower::layer::util::Identity;

use crate::{prepare_behave::{
    effect_traits::{PrepareMiddlewareEffect, PrepareStateEffect},
    StateCollector,
}, TypeNotInState};

/// [PrepareStateEffect] or [PrepareMiddlewareEffect] adding a new state type
pub struct AddState<S>(pub S);

impl<State: 'static, Service> PrepareMiddlewareEffect<Service> for AddState<State> {
    type Middleware = Identity;

    fn take(self, states: &mut StateCollector) -> Result<Self::Middleware, TypeNotInState>{
        self.take_state(states);
        Ok(Identity::new())
    }
}

impl<S: 'static> PrepareStateEffect for AddState<S> {
    fn take_state(self, states: &mut StateCollector) {
        states.insert(self.0)
    }
}

impl<S> AddState<S> {
    /// adding state
    pub fn new(state: S) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        Self(state)
    }
}

impl<S> AddState<Arc<S>> {
    /// adding state wrapped by [Arc]
    pub fn arc(state: S) -> Self
    where
        Arc<S>: Clone + Send + Sync + 'static,
    {
        Self(Arc::new(state))
    }
}
