#[allow(unused_imports)]
use std::any::type_name;
use std::{future::IntoFuture, sync::Arc};

use futures::{
    future::{join, ok, Ready},
    Future, FutureExt, TryFutureExt,
};

use crate::{
    prepare_behave::{
        effect_traits::{Prepare, PrepareStateEffect},
        StateCollector,
    },
    PrepareError,
};

/// apply all [Prepare](Prepare) task concurrently
///
/// ## Note
/// the sync part of [Prepare] will be run immediately
pub struct ConcurrentPrepareSet<C, PFut> {
    prepare_fut: PFut,
    configure: Arc<C>,
}

impl<C, PFut> ConcurrentPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<StateCollector, PrepareError>>,
{
    /// get the [Future]
    pub(crate) fn into_internal_future(self) -> PFut {
        self.prepare_fut
    }
}

impl<C, PFut> ConcurrentPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<StateCollector, PrepareError>>,

    C: 'static,
{
    /// join a [Prepare] into concurrent execute set
    ///
    /// concurrent only support state prepare
    pub fn join_state<P>(
        self,
        prepare: P,
    ) -> ConcurrentPrepareSet<C, impl Future<Output = Result<StateCollector, PrepareError>>>
    where
        P: Prepare<C>,
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
                .map_err(PrepareError::to_prepare_error::<P, _>),
        )
        .map(|(l, r)| {
            Ok({
                let mut states = l?;
                let effect = r?;
                effect.take_state(&mut states);

                states
            })
        });

        ConcurrentPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    /// join a [Prepare] without effect
    pub fn join<P: Prepare<C, Effect = ()>>(
        self,
        prepare: P,
    ) -> ConcurrentPrepareSet<C, impl Future<Output = Result<StateCollector, PrepareError>>> {
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
                .map_err(PrepareError::to_prepare_error::<P, _>),
        )
        .map(|(l, r)| {
            r?;
            l
        });

        ConcurrentPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
}

impl<C: 'static> ConcurrentPrepareSet<C, Ready<Result<StateCollector, PrepareError>>> {
    pub(crate) fn new(configure: Arc<C>) -> Self {
        Self {
            prepare_fut: ok(StateCollector::new()),
            configure,
        }
    }
}
