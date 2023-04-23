#[allow(unused_imports)]
use std::{any::type_name, mem::size_of_val};
use std::{future::IntoFuture, sync::Arc};

use futures::{
    future::{ok}, FutureExt, TryFutureExt,
};
use http_body::Body;
use tower::layer::util::{Identity, Stack};

use crate::{
    prepare_behave::{
        effect_traits::{Prepare, PrepareMiddlewareEffect, PrepareRouteEffect, PrepareStateEffect},
        EffectContainer,
    },
    ConcurrentPrepareSet, PrepareError,
};

use super::{BoxFuture, ContainerFuture, ContainerResult};

/// a set of [Prepare] task executing one by one
///
/// ## Note
/// the sync part of [Prepare] will be run immediately
pub struct SerialPrepareSet<C, T> {
    prepare_fut: BoxFuture<T>,
    configure: Arc<C>,
}

impl<C, T> SerialPrepareSet<C, T> {
    pub(crate) fn get_ref_configure(&self) -> &C {
        &self.configure
    }

    pub(crate) fn get_configure(&self) -> Arc<C> {
        Arc::clone(&self.configure)
    }
}

impl<C, R, L> SerialPrepareSet<C, ContainerResult<R, L>> {
    pub(crate) fn unwrap(self) -> (ContainerFuture<R, L>, Arc<C>) {
        (self.prepare_fut, self.configure)
    }
}

impl<C, R, L> SerialPrepareSet<C, ContainerResult<R, L>>
where
    C: 'static,
    R: 'static,
    L: 'static,
{
    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareRouteEffect]
    pub(crate) fn then_route<P, S, B>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<C, ContainerResult<(P::Effect, R), L>>
    where
        P: Prepare<C> + 'static,
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
            .and_then(|collector| collector.then_route(prepare, configure))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareStateEffect]
    pub(crate) fn then_state<P>(self, prepare: P) -> SerialPrepareSet<C, ContainerResult<R, L>>
    where
        P: Prepare<C> + 'static,
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
            .and_then(|collector| collector.then_state(prepare, configure))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareMiddlewareEffect]
    pub(crate) fn then_middleware<P, S>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<
        C,
        ContainerResult<R, Stack<<P::Effect as PrepareMiddlewareEffect<S>>::Middleware, L>>,
    >
    where
        S: 'static,
        P: Prepare<C> + 'static,
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
            .and_then(|collector| collector.then_middleware(prepare, configure))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
    /// add a [Prepare] into serially executing set
    ///
    /// without Effect
    pub(crate) fn then<P>(self, prepare: P) -> SerialPrepareSet<C, ContainerResult<R, L>>
    where
        P: Prepare<C, Effect = ()> + 'static,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();

        let prepare_fut = self
            .prepare_fut
            .and_then(|collect| {
                prepare
                    .prepare(configure)
                    .into_future()
                    .map_ok(|_| collect)
                    .map_err(|err| PrepareError::to_prepare_error::<P, _>(err))
            })
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    /// just adding a middleware
    pub(crate) fn set_middleware<M: 'static>(
        self,
        layer: M,
    ) -> SerialPrepareSet<C, ContainerResult<R, Stack<M, L>>> {
        SerialPrepareSet {
            prepare_fut: self
                .prepare_fut
                .map_ok(|effect| effect.layer(layer))
                .boxed_local(),
            configure: self.configure,
        }
    }

    /// combine concurrent set into self
    pub(crate) fn combine(
        self,
        concurrent: ConcurrentPrepareSet<C>,
    ) -> SerialPrepareSet<C, ContainerResult<R, L>> {
        let fut = concurrent.into_internal_future();

        let prepare_fut = self
            .prepare_fut
            .and_then(|container| fut.map_ok(|states| container.combine_state(states)))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
}

impl<C: 'static> SerialPrepareSet<C, ContainerResult<(), Identity>> {
    pub(crate) fn new(configure: Arc<C>) -> Self {
        Self {
            prepare_fut: ok(EffectContainer::new()).boxed_local(),
            configure,
        }
    }
}
