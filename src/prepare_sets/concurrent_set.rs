#[allow(unused_imports)]
use std::any::type_name;
use std::marker::PhantomData;
use std::{future::IntoFuture, sync::Arc};

use futures::{
    future::{join, ok},
    FutureExt, TryFutureExt,
};
use tap::Pipe;

use crate::server_prepare::PrepareDecorator;
use crate::{
    prepare_behave::{
        effect_traits::{Prepare, PrepareStateEffect},
        StateCollector,
    },
    PrepareError,
};

use super::{BoxFuture, StateContainerFuture, StateContainerResult};

/// apply all [Prepare](Prepare) task concurrently
///
/// ## Note
/// the sync part of [Prepare] will be run immediately
pub struct ConcurrentPrepareSet<C, Decorator, T = StateContainerResult> {
    prepare_fut: BoxFuture<T>,
    configure: Arc<C>,
    __phantom_decorator: PhantomData<Decorator>,
}

impl<C, Decorator> ConcurrentPrepareSet<C, Decorator> {
    /// get the [Future]
    pub(crate) fn into_internal_future(self) -> StateContainerFuture {
        self.prepare_fut
    }
}

impl<C, Decorator> ConcurrentPrepareSet<C, Decorator>
where
    C: 'static,
    Decorator: PrepareDecorator,
{
    /// join a [Prepare] into concurrent execute set
    ///
    /// concurrent only support state prepare
    pub fn join_state<P>(self, prepare: P) -> ConcurrentPrepareSet<C, Decorator>
    where
        P: Prepare<C> + 'static,
        P::Effect: PrepareStateEffect,
    {
        debug!(
            mode = "concurrently",
            action = "Adding Prepare State",
            prepare = type_name::<P>(),
        );

        let configure = Arc::clone(&self.configure);
        let prepare_fut = join(
            self.prepare_fut,
            prepare
                .prepare(configure)
                .into_future()
                .map_err(PrepareError::to_prepare_error::<P, _>)
                .pipe(Decorator::decorator),
        )
        .map(|(l, r)| {
            Ok({
                let mut states = l?;
                let effect = r?;
                effect.take_state(&mut states);

                states
            })
        })
        .boxed_local();

        ConcurrentPrepareSet {
            prepare_fut,
            configure: self.configure,
            __phantom_decorator: PhantomData,
        }
    }

    /// join a [Prepare] without effect
    pub fn join<P>(self, prepare: P) -> ConcurrentPrepareSet<C, Decorator>
    where
        P: Prepare<C, Effect = ()> + 'static,
    {
        debug!(
            mode = "concurrently",
            action = "Adding Prepare",
            prepare = type_name::<P>(),
        );

        let configure = Arc::clone(&self.configure);
        let prepare_fut = join(
            self.prepare_fut,
            prepare
                .prepare(configure)
                .into_future()
                .map_err(PrepareError::to_prepare_error::<P, _>)
                .pipe(Decorator::decorator),
        )
        .map(|(l, r)| {
            r?;
            l
        })
        .boxed_local();

        ConcurrentPrepareSet {
            prepare_fut,
            configure: self.configure,
            __phantom_decorator: PhantomData,
        }
    }
}

impl<C: 'static, Decorator> ConcurrentPrepareSet<C, Decorator> {
    pub(crate) fn new(configure: Arc<C>) -> Self {
        Self {
            prepare_fut: ok(StateCollector::new()).boxed_local(),
            configure,
            __phantom_decorator: PhantomData,
        }
    }
}
