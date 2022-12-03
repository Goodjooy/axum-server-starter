mod prepare;

use http_body::Body;
use tower::{
    layer::util::{Identity, Stack},
    ServiceBuilder,
};

use super::{
    effect_collectors::state_collector::StateCollector,
    traits::{
        prepare_middleware::MiddlewarePrepareEffect, prepare_route::PrepareRouteEffect,
        prepare_state::PrepareStateEffect,
    },
};

pub struct EffectContainer<R, L> {
    states: StateCollector,
    middleware: ServiceBuilder<L>,
    route: R,
}

impl<R, L> EffectContainer<R, L> {
    pub fn unwrap(self) -> (StateCollector, ServiceBuilder<L>, R) {
        (self.states, self.middleware, self.route)
    }
}

impl<R, L> EffectContainer<R, L> {
    pub(crate) fn combine_state(mut self, states: StateCollector) -> Self {
        self.states = self.states & states;
        self
    }
}

impl EffectContainer<(), Identity> {
    pub(crate) fn new() -> Self {
        Self {
            states: StateCollector::new(),
            middleware: ServiceBuilder::new(),
            route: (),
        }
    }
}

impl<R, L> EffectContainer<R, L> {
    pub fn set_middleware<Service, E: MiddlewarePrepareEffect<Service>>(
        self,
        effect: E,
    ) -> EffectContainer<R, Stack<E::Middleware, L>> {
        let EffectContainer {
            mut states,
            middleware,
            route,
        } = self;

        let middleware = middleware.layer(effect.take(&mut states));

        EffectContainer {
            states,
            middleware,
            route,
        }
    }

    pub(crate) fn layer<M>(self, layer: M) -> EffectContainer<R, Stack<M, L>> {
        let EffectContainer {
            states,
            middleware,
            route,
        } = self;

        let middleware = middleware.layer(layer);

        EffectContainer {
            states,
            middleware,
            route,
        }
    }

    pub fn set_state<E>(mut self, effect: E) -> EffectContainer<R, L>
    where
        E: PrepareStateEffect,
    {
        effect.take_state(&mut self.states);
        self
    }

    pub fn set_route<S, B, E>(self, effect: E) -> EffectContainer<(E, R), L>
    where
        B: Body + Send + 'static,
        E: PrepareRouteEffect<S, B>,
        S: Clone + Send + 'static + Sync,
    {
        let EffectContainer {
            states,
            middleware,
            route,
        } = self;

        let route = (effect, route);

        EffectContainer {
            states,
            middleware,
            route,
        }
    }
}
