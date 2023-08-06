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
pub struct ConcurrentPrepareSet<'d ,C, Decorator, T = StateContainerResult> {
    prepare_fut: BoxFuture<T>,
    configure: Arc<C>,
    decorator: &'d Decorator,
}

impl<'d,C, Decorator> ConcurrentPrepareSet<'d,C, Decorator> {
    /// get the [Future]
    pub(crate) fn into_internal_future(self) -> StateContainerFuture {
        self.prepare_fut
    }
}

impl<'d,C, Decorator> ConcurrentPrepareSet<'d,C, Decorator>
where
    C: 'static,
    Decorator: PrepareDecorator,
{
    /// join a [Prepare] into concurrent execute set
    ///
    /// concurrent only support state prepare
    pub fn join_state<P>(self, prepare: P) -> ConcurrentPrepareSet<'d,C, Decorator>
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
                .pipe(|fut|self.decorator.prepare_decorator::<C,P,_>(fut)),
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
            decorator:self.decorator
        }
    }

    /// join a [Prepare] without effect
    pub fn join<P>(self, prepare: P) -> ConcurrentPrepareSet<'d,C, Decorator>
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
                .pipe(|fut|self.decorator.prepare_decorator::<C,P,_>(fut)),
        )
        .map(|(l, r)| {
            r?;
            l
        })
        .boxed_local();

        ConcurrentPrepareSet {
            prepare_fut,
            configure: self.configure,
            decorator:self.decorator
        }
    }
}

impl<'d,C: 'static, Decorator> ConcurrentPrepareSet<'d,C, Decorator> {
    pub(crate) fn new(configure: Arc<C>,decorator:&'d Decorator) -> Self {
        Self {
            prepare_fut: ok(StateCollector::new()).boxed_local(),
            configure,
            decorator,
        }
    }
}
