#[allow(unused_imports)]
use std::{any::type_name, mem::size_of_val};
use std::{future::IntoFuture, sync::Arc};

use futures::{
    future::{ok, Ready},
    Future, TryFutureExt,
};
use http_body::Body;
use tower::layer::util::{Identity, Stack};

use crate::{
    debug,
    prepare_behave::{
        effect_traits::{Prepare, PrepareMiddlewareEffect, PrepareRouteEffect, PrepareStateEffect},
        EffectContainer, StateCollector,
    },
    ConcurrentPrepareSet, PrepareError,
};

/// a set of [Prepare] task executing one by one
///
/// ## Note
/// the sync part of [Prepare] will be run immediately
pub struct SerialPrepareSet<C, PFut> {
    prepare_fut: PFut,
    configure: Arc<C>,
}

impl<C, PFut> SerialPrepareSet<C, PFut> {
    pub(crate) fn get_ref_configure(&self) -> &C {
        &self.configure
    }

    pub(crate) fn get_configure(&self) -> Arc<C> {
        Arc::clone(&self.configure)
    }
}

impl<C, PFut, R, L> SerialPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<EffectContainer<R, L>, PrepareError>>,
{
    /// get the [Future] with return [IntoFallibleEffect](crate::IntoFallibleEffect)
    pub fn to_prepared_effect(self) -> PFut {
        self.prepare_fut
    }

    pub(crate) fn unwrap(self) -> (PFut, Arc<C>) {
        (self.prepare_fut, self.configure)
    }
}

impl<C, PFut, R, L> SerialPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<EffectContainer<R, L>, PrepareError>>,
    C: 'static,
{
    /// add a [Prepare] into serially executing set
    pub fn then_route<P, S, B>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<
        C,
        impl Future<Output = Result<EffectContainer<(P::Effect, R), L>, PrepareError>>,
    >
    where
        P: Prepare<C>,
        P::Effect: PrepareRouteEffect<S, B>,
        P::Error: 'static,
        R: PrepareRouteEffect<S, B>,
        B: Body + Send + 'static,
        S: Clone + Send + 'static + Sync,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare Route",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();

        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_route(prepare, configure));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    pub fn then_state<P>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<C, impl Future<Output = Result<EffectContainer<R, L>, PrepareError>>>
    where
        P: Prepare<C>,
        P::Effect: PrepareStateEffect,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare State",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();

        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_state(prepare, configure));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    pub fn then_middleware<P, S>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<
        C,
        impl Future<
            Output = Result<
                EffectContainer<R, Stack<<P::Effect as PrepareMiddlewareEffect<S>>::Middleware, L>>,
                PrepareError,
            >,
        >,
    >
    where
        P: Prepare<C>,
        P::Effect: PrepareMiddlewareEffect<S>,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare Middleware",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();

        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_middleware(prepare, configure));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    pub fn then<P>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<C, impl Future<Output = Result<EffectContainer<R, L>, PrepareError>>>
    where
        P: Prepare<C, Effect = ()>,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();

        let prepare_fut = self.prepare_fut.and_then(|collect| {
            prepare
                .prepare(configure)
                .into_future()
                .map_ok(|_| collect)
                .map_err(|err| PrepareError::to_prepare_error::<P, _>(err))
        });

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    pub fn set_middleware<M>(
        self,
        layer: M,
    ) -> SerialPrepareSet<
        C,
        impl Future<Output = Result<EffectContainer<R, Stack<M, L>>, PrepareError>>,
    > {
        SerialPrepareSet {
            prepare_fut: self.prepare_fut.map_ok(|effect| effect.layer(layer)),
            configure: self.configure,
        }
    }

    pub fn combine<ConcurrentFut>(
        self,
        concurrent: ConcurrentPrepareSet<C, ConcurrentFut>,
    ) -> SerialPrepareSet<C, impl Future<Output = Result<EffectContainer<R, L>, PrepareError>>>
    where
        ConcurrentFut: Future<Output = Result<StateCollector, PrepareError>>,
    {
        let fut = concurrent.to_prepared_effect();

        let prepare_fut = self
            .prepare_fut
            .and_then(|container| fut.map_ok(|states| container.combine_state(states)));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
}

impl<C: 'static> SerialPrepareSet<C, Ready<Result<EffectContainer<(), Identity>, PrepareError>>> {
    pub(crate) fn new(configure: Arc<C>) -> Self {
        Self {
            prepare_fut: ok(EffectContainer::new()),
            configure,
        }
    }
}
