use std::future::IntoFuture;
use std::sync::Arc;

use http_body::Body;
use tap::Pipe;
use tower::layer::util::Stack;
use futures::TryFutureExt;
use crate::{prepare_behave::traits::{
    prepare_middleware::PrepareMiddlewareEffect, prepare_route::PrepareRouteEffect,
    prepare_state::PrepareStateEffect, Prepare,
}, PrepareDecorator, PrepareError};

use super::EffectContainer;

impl<R, L> EffectContainer<R, L> {
    pub(crate) async fn then_route<D,S, B, C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<EffectContainer<(P::Effect, R), L>, PrepareError>
    where
    D:PrepareDecorator,
        C: 'static,
        P: Prepare<C>,
        B: Body + 'static + Send,
        P::Effect: PrepareRouteEffect<S, B>,
        S: Clone + Send + 'static + Sync,
    {
        Ok(self.set_route(
            prepare
                .prepare(configure)
                .into_future()
                .map_err(|err| PrepareError::to_prepare_error::<P, _>(err))
                .pipe(D::decorator)
                .await
                ?,
        ))
    }

    pub(crate) async fn then_state<D,C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<Self, PrepareError>
    where
        D:PrepareDecorator,
        C: 'static,
        P: Prepare<C>,
        P::Effect: PrepareStateEffect,
    {
        Ok(self.set_state(
            prepare
                .prepare(configure)
                .into_future()
                .map_err(PrepareError::to_prepare_error::<P, _>)
                .pipe(D::decorator)
                .await
                ?,
        ))
    }

    pub(crate) async fn then_middleware<D,S, C, P>(
        self,
        prepare: P,
        configure: Arc<C>,
    ) -> Result<
        EffectContainer<R, Stack<<P::Effect as PrepareMiddlewareEffect<S>>::Middleware, L>>,
        PrepareError,
    >
    where
        D:PrepareDecorator,
        C: 'static,
        P: Prepare<C>,
        P::Effect: PrepareMiddlewareEffect<S>,
    {
        Ok(self.set_middleware(
            prepare
                .prepare(configure)
                .into_future()
                .map_err(PrepareError::to_prepare_error::<P, _>)
                .pipe(D::decorator)
                .await
                ?,
        ))
    }
}
