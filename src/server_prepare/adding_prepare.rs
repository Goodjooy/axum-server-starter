use futures::{future::Ready, Future};
use tower::layer::util::Stack;

use crate::{
    debug,
    prepare_behave::{
        effect_traits::{MiddlewarePrepareEffect, Prepare, PrepareRouteEffect, PrepareStateEffect},
        EffectContainer, StateCollector,
    },
    ConcurrentPrepareSet, PrepareError, ServerPrepare,
};

impl<C: 'static, FutEffect, Log, State, Graceful>
    ServerPrepare<C, FutEffect, Log, State, Graceful>
{
    /// adding a set of [Prepare] executing concurrently
    ///
    /// # Note
    ///
    /// [Prepare] set added by different [Self::append_concurrent] will be execute serially
    pub fn prepare_concurrent<F, Fut, R, Li>(
        self,
        concurrent: F,
    ) -> ServerPrepare<
        C,
        impl Future<Output = Result<EffectContainer<R, Li>, PrepareError>>,
        Log,
        State,
        Graceful,
    >
    where
        F: FnOnce(
                ConcurrentPrepareSet<C, Ready<Result<StateCollector, PrepareError>>>,
            ) -> ConcurrentPrepareSet<C, Fut>
            + 'static,
        Fut: Future<Output = Result<StateCollector, PrepareError>>,
        FutEffect: Future<Output = Result<EffectContainer<R, Li>, PrepareError>>,
    {
        let prepares = self.span.in_scope(|| {
            debug!(mode = "Concurrent", action = "Add Prepare");
            let concurrent_set = ConcurrentPrepareSet::new(self.prepares.get_configure());
            self.prepares.combine(concurrent(concurrent_set))
        });
        ServerPrepare::new(prepares, self.graceful, self.span)
    }

    /// adding a [Prepare] apply effect on route
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_route<P, R, LayerInner, B, S>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        impl Future<
            Output = Result<
                EffectContainer<impl PrepareRouteEffect<S, B>, LayerInner>,
                PrepareError,
            >,
        >,
        Log,
        State,
        Graceful,
    >
    where
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,

        P: Prepare<C>,
        P::Effect: PrepareRouteEffect<S, B>,
        R: PrepareRouteEffect<S, B>,
        B: http_body::Body + Send + 'static,
        S: Clone + Send + 'static + Sync,
    {
        let prepares = self.span.in_scope(|| {
            debug!(
                mode = "Serial",
                action = "Add Prepare Route",
                prepare = core::any::type_name::<P>()
            );
            self.prepares.then_route(prepare)
        });

        ServerPrepare::new(prepares, self.graceful, self.span)
    }
    /// adding a [Prepare] adding effect
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_state<P, R, LayerInner>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        impl Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,
        Log,
        State,
        Graceful,
    >
    where
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,

        P: Prepare<C>,
        P::Effect: PrepareStateEffect,
    {
        let prepares = self.span.in_scope(|| {
            debug!(
                mode = "Serial",
                action = "Add Prepare State",
                prepare = core::any::type_name::<P>()
            );
            self.prepares.then_state(prepare)
        });

        ServerPrepare::new(prepares, self.graceful, self.span)
    }

    /// adding a [Prepare] apply  effect on State and Middleware
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_middleware<S, P, R, LayerInner>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        impl Future<
            Output = Result<
                EffectContainer<
                    R,
                    Stack<<P::Effect as MiddlewarePrepareEffect<S>>::Middleware, LayerInner>,
                >,
                PrepareError,
            >,
        >,
        Log,
        State,
        Graceful,
    >
    where
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,

        P: Prepare<C>,
        P::Effect: MiddlewarePrepareEffect<S>,
    {
        let prepares = self.span.in_scope(|| {
            debug!(
                mode = "Serial",
                action = "Add Prepare Middleware",
                prepare = core::any::type_name::<P>()
            );
            self.prepares.then_middleware(prepare)
        });

        ServerPrepare::new(prepares, self.graceful, self.span)
    }

    pub fn prepare< P, R, LayerInner>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        impl Future<
            Output = Result<
                EffectContainer<
                    R,
                    LayerInner,
                >,
                PrepareError,
            >,
        >,
        Log,
        State,
        Graceful,
    >
    where
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,

        P: Prepare<C,Effect = ()>,
    {
        
        let prepares = self.span.in_scope(|| {
            debug!(
                mode = "Serial",
                action = "Add Prepare Middleware",
                prepare = core::any::type_name::<P>()
            );
            self.prepares.then(prepare)
        });

        ServerPrepare::new(prepares, self.graceful, self.span)
    }
}
