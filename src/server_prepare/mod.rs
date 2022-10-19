mod error;
use std::{
    convert::Infallible,
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
    future::{join, ok, Ready},
    Future, FutureExt, TryFutureExt,
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

use crate::{fn_prepare, server_ready::ServerReady, PrepareHandler};

use self::error::{flatten_result, to_prepare_error};
pub use self::{
    configure::{ConfigureServerEffect, LoggerInitialization, ServeAddress},
    error::PrepareError,
    prepare::{
        ExtensionEffect, ExtensionManage, GracefulEffect, Prepare, PreparedEffect, RouteEffect,
        ServerEffect,
    },
};
mod configure;
mod prepare;

pub struct NoLog;
pub struct LogInit;

/// type for prepare starting
pub struct ServerPrepare<C, L, FutEffect, Log = LogInit> {
    config: Arc<C>,
    prepares: FutEffect,
    middleware: ServiceBuilder<L>,
    _phantom: PhantomData<Log>,
}

impl<C, L, FutEffect, Log> ServerPrepare<C, L, FutEffect, Log> {
    pub fn new(config: Arc<C>, prepares: FutEffect, middleware: ServiceBuilder<L>) -> Self {
        Self {
            config,
            prepares,
            middleware,
            _phantom: PhantomData,
        }
    }
}

impl<C, L, FutEffect> ServerPrepare<C, L, FutEffect, NoLog>
where
    C: LoggerInitialization,
{
    /// init the logger of this [ServerPrepare] ,require C impl [LoggerInitialization]
    pub fn init_logger(self) -> Result<ServerPrepare<C, L, FutEffect, LogInit>, C::Error> {
        self.config.init_logger()?;
        Ok(ServerPrepare::new(
            self.config,
            self.prepares,
            self.middleware,
        ))
    }
}

impl<C> ServerPrepare<C, Identity, Ready<Result<(), PrepareError>>, NoLog> {
    pub fn with_config(config: C) -> Self
    where
        C: ServeAddress,
    {
        ServerPrepare::new(Arc::new(config), ok(()), ServiceBuilder::new())
    }
}

impl<C: 'static, L, FutEffect, Effect, Log> ServerPrepare<C, L, FutEffect, Log>
where
    FutEffect: Future<Output = Result<Effect, PrepareError>>,
    Effect: PreparedEffect,
{
    /// adding a [Prepare]
    /// 
    /// ## Note
    /// 
    /// the [Prepare] task will be waiting at the same time.
    /// 
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn append<P>(
        self,
        prepare: P,
    ) -> ServerPrepare<C, L, impl Future<Output = Result<impl PreparedEffect, PrepareError>>>
    where
        FutEffect: Future,
        P: Prepare<C>,
    {
        let task = prepare
            .prepare(Arc::clone(&self.config))
            .map_err(to_prepare_error::<P, _>);

        let prepares = join(self.prepares, task).map(flatten_result);

        ServerPrepare::new(self.config, prepares, self.middleware)
    }
    /// adding a function-style [Prepare]
    pub fn append_fn<F, Args>(
        self,
        func: F,
    ) -> ServerPrepare<C, L, impl Future<Output = Result<impl PreparedEffect, PrepareError>>>
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
    pub fn with_global_middleware<M>(self, layer: M) -> ServerPrepare<C, Stack<M, L>, FutEffect> {
        ServerPrepare::new(self.config, self.prepares, self.middleware.layer(layer))
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
        C: ServeAddress + ConfigureServerEffect,
        ServiceBuilder<L>: Layer<Route>,
        <ServiceBuilder<L> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response = Response<NewResBody>, Error = Infallible>
            + 'static,
        <<ServiceBuilder<L> as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
        NewResBody: http_body::Body<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
    {
        let (extension_effect, route_effect, graceful_effect, server_effect) =
            self.prepares.await?.split_effect();

        let router = Router::new()
            // apply prepare effect on router
            .pipe(|router| route_effect.add_router(router))
            // apply prepare extension
            .pipe(|router| extension_effect.apply_extension(router))
            // adding middleware
            .pipe(|router| router.layer(self.middleware));

        let graceful = graceful_effect.set_graceful();

        let server = server::Server::bind(&ServeAddress::get_address(&*self.config).into())
            // apply effect config server
            .pipe(|server| server_effect.config_serve(server))
            // apply configure config server
            .pipe(|server| self.config.effect_server(server))
            .serve(router.into_make_service());

        Ok(match graceful {
            Some(fut) => ServerReady::Graceful(server.with_graceful_shutdown(fut)),
            None => ServerReady::Server(server),
        })
    }
}
