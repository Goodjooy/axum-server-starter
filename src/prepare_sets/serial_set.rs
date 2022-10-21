use std::sync::Arc;

use futures::{
    future::{ok, Ready},
    Future, TryFutureExt,
};

use crate::{
    EffectsCollector, ExtensionEffect, GracefulEffect, Prepare, PrepareError, PreparedEffect,
    RouteEffect, ServerEffect,
};

pub struct SerialPrepareSet<C, PFut> {
    prepare_fut: PFut,
    configure: Arc<C>,
}

impl<C, PFut, E> SerialPrepareSet<C, PFut>
where
    PFut: Future<Output = Result<E, PrepareError>>,
    E: PreparedEffect,
{
    pub fn to_prepared_effect(self) -> PFut {
        self.prepare_fut
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
    ) -> SerialPrepareSet<C, impl Future<Output = Result<impl PreparedEffect, PrepareError>>> {
        let configure = Arc::clone(&self.configure);

        let prepare_fut = self
            .prepare_fut
            .and_then(|collector| collector.with_prepare(prepare, configure));

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
