use tower::layer::util::Identity;

use crate::prepare_behave::{
    effect_traits::{PrepareMiddlewareEffect, PrepareStateEffect},
    StateCollector,
};

pub struct AddState<S>(pub S);

impl<State: 'static, Service> PrepareMiddlewareEffect<Service> for AddState<State> {
    type Middleware = Identity;

    fn take(self, states: &mut StateCollector) -> Self::Middleware {
        self.take_state(states);
        Identity::new()
    }
}

impl<S: 'static> PrepareStateEffect for AddState<S> {
    fn take_state(self, states: &mut StateCollector) {
        states.insert(self.0)
    }
}

impl<S> AddState<S> {
    pub fn new(state: S) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        Self(state)
    }
}
