use std::sync::Arc;

use futures::{
    future::{ok, Ready},
    Future, TryFutureExt,
};

use crate::{
    prepared_effect::CombineEffects, EffectsCollector, ExtensionEffect, GracefulEffect, Prepare,
    PrepareError, PreparedEffect, RouteEffect, ServerEffect,
};

pub struct SerialPrepareSet<C, PFut> {
    prepare_fut: PFut,
    configure: Arc<C>,
}

impl<C, PFut> SerialPrepareSet<C, PFut> {
    pub(crate) fn get_ref_configure(&self)->&C{
        &self.configure
    }

    pub(crate) fn get_configure(&self) -> Arc<C> {
        Arc::clone(&self.configure)
    }
}

impl<C, PFut, E> SerialPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<E, PrepareError>>,
    E: PreparedEffect,
{
    pub fn to_prepared_effect(self) -> PFut {
        self.prepare_fut
    }

    pub(crate) fn unwrap(self)->(PFut,Arc<C>){
        (self.prepare_fut,self.configure)
    }
}

impl<C, PFut, R, S, G, E> SerialPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<EffectsCollector<R, G, E, S>, PrepareError>>,
    R: RouteEffect,
    S: ServerEffect,
    E: ExtensionEffect,
    G: GracefulEffect,
    C: 'static,
{
    pub fn then<P: Prepare<C>>(
        self,
        prepare: P,
    ) -> SerialPrepareSet<
        C,
        impl Future<Output = Result<CombineEffects<R, G, E, S, P::Effect>, PrepareError>>,
    > {
        let configure = Arc::clone(&self.configure);

        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.with_prepare(prepare, configure));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }

    pub(crate) fn then_fut_effect<Fut, Effect>(
        self,
        fut: Fut,
    ) -> SerialPrepareSet<
        C,
        impl Future<Output = Result<CombineEffects<R, G, E, S, Effect>, PrepareError>>,
    >
    where
        Fut: Future<Output = Result<Effect, PrepareError>>,
        Effect: PreparedEffect,
    {
        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.with_future_effect(fut));

        SerialPrepareSet {
            prepare_fut,
            configure: self.configure,
        }
    }
}

impl<C: 'static> SerialPrepareSet<C, Ready<Result<EffectsCollector, PrepareError>>> {
    pub(crate) fn new(configure: Arc<C>) -> Self {
        Self {
            prepare_fut: ok(EffectsCollector::new()),
            configure,
        }
    }

}
