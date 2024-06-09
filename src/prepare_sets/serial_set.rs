#[allow(unused_imports)]
use std::{any::type_name, mem::size_of_val};
use std::{future::IntoFuture, sync::Arc};

use futures::{future::ok, FutureExt, TryFutureExt};
use tap::Pipe;
use tower::layer::util::{Identity, Stack};

use crate::prepare_behave::effect_contain::{BaseRouter, TestRouter};
use crate::server_prepare::PrepareDecorator;
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
pub struct SerialPrepareSet<C, T, Decorator> {
    prepare_fut: BoxFuture<T>,
    configure: Arc<C>,
    decorator: Arc<Decorator>,
}

impl<C, T, Decorator> SerialPrepareSet<C, T, Decorator> {
    pub(crate) fn get_ref_configure(&self) -> &C {
        &self.configure
    }

    pub(crate) fn get_configure(&self) -> Arc<C> {
        Arc::clone(&self.configure)
    }
    pub(crate) fn change_decorator<D: PrepareDecorator>(
        self,
        decorator: D,
    ) -> SerialPrepareSet<C, T, D> {
        SerialPrepareSet {
            prepare_fut: self.prepare_fut,
            configure: self.configure,
            decorator: Arc::new(decorator),
        }
    }

    pub(crate) fn get_decorator(&self) -> Arc<Decorator> {
        self.decorator.clone()
    }
}

impl<C, R, L, Decorator> SerialPrepareSet<C, ContainerResult<R, L>, Decorator> {
    pub(crate) fn unwrap(self) -> (ContainerFuture<R, L>, Arc<C>) {
        (self.prepare_fut, self.configure)
    }
}

type MiddlewareContainerResult<R, L, P, S, C> = ContainerResult<
    R,
    Stack<<<P as Prepare<C>>::Effect as PrepareMiddlewareEffect<S>>::Middleware, L>,
>;

type ThenRouterPrepareRet<C, P, R, L, Decorator> =
    SerialPrepareSet<C, ContainerResult<BaseRouter<(<P as Prepare<C>>::Effect, R)>, L>, Decorator>;

impl<C, R, L, Decorator> SerialPrepareSet<C, ContainerResult<BaseRouter<R>, L>, Decorator>
where
    C: 'static,
    R: 'static,
    L: 'static,
    Decorator: PrepareDecorator,
{
    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareRouteEffect]
    pub(crate) fn then_route<P, S>(self, prepare: P) -> ThenRouterPrepareRet<C, P, R, L, Decorator>
    where
        P: Prepare<C> + 'static,
        P::Effect: PrepareRouteEffect<S>,
        P::Error: 'static,
        R: PrepareRouteEffect<S>,
        S: Clone + Send + 'static + Sync,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare Route",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();
        let decorator = self.get_decorator();
        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_route(prepare, configure, decorator))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator: self.decorator,
        }
    }
}

impl<C, R, L, Decorator> SerialPrepareSet<C, ContainerResult<R, L>, Decorator>
where
    C: 'static,
    R: 'static,
    L: 'static,
    Decorator: PrepareDecorator,
{

    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareStateEffect]
    pub(crate) fn then_state<P>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<C, ContainerResult<R, L>, Decorator>
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
        let decorator = self.get_decorator();
        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_state(prepare, configure, decorator))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator: self.decorator,
        }
    }
    /// add a [Prepare] into serially executing set
    ///
    /// with the [PrepareMiddlewareEffect]
    pub(crate) fn then_middleware<P, S>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<C, MiddlewareContainerResult<R, L, P, S, C>, Decorator>
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
        let decorator = self.get_decorator();
        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.then_middleware(prepare, configure, decorator))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator: self.decorator,
        }
    }
    /// add a [Prepare] into serially executing set
    ///
    /// without Effect
    pub(crate) fn then<P>(self, prepare: P) -> SerialPrepareSet<C, ContainerResult<R, L>, Decorator>
    where
        P: Prepare<C, Effect = ()> + 'static,
    {
        debug!(
            mode = "serially",
            action = "Adding Prepare",
            prepare = type_name::<P>(),
        );

        let configure = self.get_configure();
        let decorator = self.get_decorator();

        let prepare_fut = self
            .prepare_fut
            .and_then(|collect| {
                prepare
                    .prepare(configure)
                    .into_future()
                    .map_err(|err| PrepareError::to_prepare_error::<P, _>(err))
                    .pipe(move |fut| decorator.prepare_decorator::<C, P, _>(fut))
                    .map_ok(|_| collect)
            })
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator: self.decorator,
        }
    }

    /// just adding a middleware
    pub(crate) fn set_middleware<M: 'static>(
        self,
        layer: M,
    ) -> SerialPrepareSet<C, ContainerResult<R, Stack<M, L>>, Decorator> {
        SerialPrepareSet {
            prepare_fut: self
                .prepare_fut
                .map_ok(|effect| effect.layer(layer))
                .boxed_local(),
            configure: self.configure,
            decorator: self.decorator,
        }
    }

    /// combine concurrent set into self
    pub(crate) fn combine(
        self,
        concurrent: ConcurrentPrepareSet<'_, C, Decorator>,
    ) -> SerialPrepareSet<C, ContainerResult<R, L>, Decorator> {
        let fut = concurrent.into_internal_future();

        let prepare_fut = self
            .prepare_fut
            .and_then(|container| fut.map_ok(|states| container.combine_state(states)))
            .boxed_local();

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator: self.decorator,
        }
    }
}

impl<C: 'static, Decorator> SerialPrepareSet<C, ContainerResult<BaseRouter<()>, Identity>, Decorator> {
    pub(crate) fn new(configure: Arc<C>, decorator: Decorator) -> Self {
        Self {
            prepare_fut: ok(EffectContainer::new()).boxed_local(),
            configure,
            decorator: Arc::new(decorator),
        }
    }
}
impl<C: 'static, Decorator> SerialPrepareSet<C, ContainerResult<TestRouter, Identity>, Decorator> {
    pub(crate) fn new_test(configure: Arc<C>, decorator: Decorator) -> Self {
        Self {
            prepare_fut: ok(EffectContainer::new_test()).boxed_local(),
            configure,
            decorator: Arc::new(decorator),
        }
    }
}
