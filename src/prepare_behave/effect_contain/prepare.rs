use std::sync::Arc;

use http_body::Body;
use tower::layer::util::Stack;

use crate::{
    prepare_behave::traits::{
        prepare_middleware::MiddlewarePrepareEffect, prepare_route::PrepareRouteEffect,
        prepare_state::PrepareStateEffect, Prepare,
    },
    PrepareError,
};

use super::EffectContainer;

impl<R, L> EffectContainer<R, L> {
    pub(crate) async fn then_route<S,B, C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<EffectContainer<(P::Effect, R), L>, PrepareError>
    where
        C: 'static,
        P: Prepare<C>,
        B: Body + 'static + Send,
        P::Effect: PrepareRouteEffect<S,B>,
        S:Clone + Send +'static + Sync
    {
        Ok(self.set_route(
            prepare
                .prepare(configure)
                .await
                .map_err(|err| PrepareError::to_prepare_error::<P, _>(err))?,
        ))
    }

    pub(crate) async fn then_state<C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<Self, PrepareError>
    where
        C: 'static,
        P: Prepare<C>,
        P::Effect: PrepareStateEffect,
    {
        Ok(self.set_state(
            prepare
                .prepare(configure)
                .await
                .map_err(PrepareError::to_prepare_error::<P, _>)?,
        ))
    }

    pub(crate) async fn then_middleware<S, C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<
        EffectContainer<R, Stack<<P::Effect as MiddlewarePrepareEffect<S>>::Middleware, L>>,
        PrepareError,
    >
    where
        C: 'static,
        P: Prepare<C>,
        P::Effect: MiddlewarePrepareEffect<S>,
    {
        Ok(self.set_middleware(
            prepare
                .prepare(configure)
                .await
                .map_err(PrepareError::to_prepare_error::<P, _>)?,
        ))
    }
}
