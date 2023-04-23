use tower::layer::util::Stack;

use crate::{
    prepare_behave::{
        effect_traits::{Prepare, PrepareMiddlewareEffect, PrepareRouteEffect, PrepareStateEffect},
    },
    prepare_sets::ContainerResult,
    ConcurrentPrepareSet, ServerPrepare,
};

impl<C: 'static, Log, State, Graceful, Ri: 'static, Li: 'static>
    ServerPrepare<C, ContainerResult<Ri, Li>, Log, State, Graceful>
{
    /// adding a set of [Prepare] executing concurrently
    ///
    /// # Note
    ///
    /// [Prepare] set added by different [Self::prepare_concurrent] will be execute serially
    pub fn prepare_concurrent<F>(
        self,
        concurrent: F,
    ) -> ServerPrepare<C, ContainerResult<Ri, Li>, Log, State, Graceful>
    where
        F: FnOnce(ConcurrentPrepareSet<C>) -> ConcurrentPrepareSet<C> + 'static,
    {
        let prepares = self.span.in_scope(|| {
            debug!(mode = "Concurrent", action = "Add Prepare");
            let concurrent_set = ConcurrentPrepareSet::new(self.prepares.get_configure());
            self.prepares.combine(concurrent(concurrent_set))
        });
        ServerPrepare::new(prepares, self.graceful, self.span)
    }

    /// adding a [Prepare] apply effect on [**Router**](axum::Router)
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_route<P, B, S>(
        self,
        prepare: P,
    ) -> ServerPrepare<C, ContainerResult<(P::Effect, Ri), Li>, Log, State, Graceful>
    where
        P: Prepare<C> + 'static,
        P::Effect: PrepareRouteEffect<S, B>,
        Ri: PrepareRouteEffect<S, B>,
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
    /// adding a [Prepare] adding effect on **State**
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_state<P>(
        self,
        prepare: P,
    ) -> ServerPrepare<C, ContainerResult<Ri, Li>, Log, State, Graceful>
    where
        P: Prepare<C> + 'static,
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

    /// adding a [Prepare] apply  effect on **State** and **Middleware**
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be execute one by one.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn prepare_middleware<S, P>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        ContainerResult<Ri, Stack<<P::Effect as PrepareMiddlewareEffect<S>>::Middleware, Li>>,
        Log,
        State,
        Graceful,
    >
    where
        S: 'static,
        P: Prepare<C> + 'static,
        P::Effect: PrepareMiddlewareEffect<S>,
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

    /// adding a [Prepare] without effect
    pub fn prepare<P>(
        self,
        prepare: P,
    ) -> ServerPrepare<C, ContainerResult<Ri, Li>, Log, State, Graceful>
    where
        P: Prepare<C, Effect = ()> + 'static,
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
