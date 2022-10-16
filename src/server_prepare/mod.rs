mod error;
use std::{
    any::type_name,
    convert::Infallible,
    error::Error,
    future::IntoFuture,
    marker::{PhantomData, Send},
    sync::Arc,
};

use axum::{
    body::Bytes,
    routing::{IntoMakeService, Route},
    BoxError, Router,
};

use futures::{
    future::{ready, Ready},
    Future, TryFutureExt,
};

use hyper::{
    server::{self, conn::AddrIncoming},
    Body, Request, Response,
};
use tap::Pipe;
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

use crate::{fn_prepare, server_ready::ServerReady, IntoFallibleEffect, PrepareHandler};

use self::error::PrepareError;
pub use self::{
    prepare::{BoxPreparedEffect, ExtensionManage, Prepare, PreparedEffect},
    serve_bind::{ServeAddress, ServerEffect},
};
mod prepare;
mod serve_bind;

/// type for prepare starting
pub struct ServerPrepare<C, L, FutEffect, Effect> {
    config: Arc<C>,
    prepares: FutEffect,
    middleware: ServiceBuilder<L>,

    _phantom: PhantomData<Effect>,
}

impl<C> ServerPrepare<C, Identity, Ready<Result<(), PrepareError>>, ()> {
    pub fn with_config(config: C) -> Self
    where
        C: ServeAddress,
    {
        ServerPrepare {
            config: Arc::new(config),
            prepares: ready(Ok(())),
            middleware: ServiceBuilder::new(),
            _phantom: PhantomData,
        }
    }
}

impl<C: 'static, L, FutEffect, Effect> ServerPrepare<C, L, FutEffect, Effect>
where
    FutEffect: Future<Output = Result<Effect, PrepareError>>,
    Effect: PreparedEffect,
{
    /// adding a [Prepare]
    pub fn append<P>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        L,
        impl Future<Output = Result<(Effect, P::Effect), PrepareError>>,
        (Effect, P::Effect),
    >
    where
        FutEffect: Future,
        P: Prepare<C>,
    {
        let task = prepare
            .prepare(Arc::clone(&self.config))
            .map_err(error_mapper::<P, P::Error>);

        let prepares = self
            .prepares
            .and_then(|v| task.map_ok(|t_effect| (v, t_effect)));

        ServerPrepare {
            config: self.config,
            prepares,
            middleware: self.middleware,
            _phantom: PhantomData,
        }
    }
    /// adding a function-style [Prepare]
    pub fn append_fn<F, Args>(
        self,
        func: F,
    ) -> ServerPrepare<
        C,
        L,
        impl Future<
            Output = Result<(Effect, <F::IntoEffect as IntoFallibleEffect>::Effect), PrepareError>,
        >,
        (Effect, <F::IntoEffect as IntoFallibleEffect>::Effect),
    >
    where
        FutEffect: Future,
        F: PrepareHandler<Args, C>,
    {
        self.append(fn_prepare(func))
    }
    /// adding global middleware
    ///
    /// ## note
    /// before call [Self::prepare_start] make sure the [Service::Response] is meet the
    /// axum requirement
    pub fn with_global_middleware<M>(
        self,
        layer: M,
    ) -> ServerPrepare<C, Stack<M, L>, FutEffect, Effect> {
        ServerPrepare {
            middleware: self.middleware.layer(layer),
            config: self.config,
            prepares: self.prepares,
            _phantom: PhantomData,
        }
    }
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
    pub async fn prepare_start<NewResBody>(
        self,
    ) -> Result<
        ServerReady<
            AddrIncoming,
            IntoMakeService<Router<Body>>,
            impl IntoFuture<Output = Result<(), hyper::Error>>,
        >,
        PrepareError,
    >
    where
        C: ServeAddress + ServerEffect,
        ServiceBuilder<L>: Layer<Route>,
        <ServiceBuilder<L> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response = Response<NewResBody>, Error = Infallible>
            + 'static,
        <<ServiceBuilder<L> as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
        NewResBody: http_body::Body<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        let mut effects = self.prepares.await?;

        let router = Router::new()
            // apply prepare effect on router
            .pipe(|router| effects.add_router(router))
            // apply prepare extension
            .pipe(|router| effects.apply_extension(router))
            // adding middleware
            .pipe(|router| router.layer(self.middleware));

        let graceful = effects.set_graceful();

        let server = server::Server::bind(&ServeAddress::get_address(&*self.config).into())
            // apply effect config server
            .pipe(|server| effects.config_serve(server))
            // apply configure config server
            .pipe(|server| self.config.effect_server(server))
            .serve(router.into_make_service());

        Ok(match graceful {
            Some(fut) => ServerReady::Graceful(server.with_graceful_shutdown(fut)),
            None => ServerReady::Server(server),
        })
    }
}

pub fn error_mapper<P, E: Error + 'static>(err: E) -> PrepareError {
    PrepareError::new(type_name::<P>(), Box::new(err))
}
