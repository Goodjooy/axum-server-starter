mod error;
use std::{
    convert::Infallible,
    future::IntoFuture,
    marker::{PhantomData, Send},
    sync::Arc,
};

use axum::{body::Bytes, routing::Route, BoxError, Router};

use futures::{future::Ready, Future};

use hyper::{server, Body, Request, Response};
use tap::Pipe;
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

use crate::{
    fn_prepare, prepared_effect::CombineEffects, server_ready::ServerReady, ConcurrentPrepareSet,
    EffectsCollector, FnPrepare, PrepareHandler, SerialPrepareSet,
};

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
    prepares: SerialPrepareSet<C, FutEffect>,
    middleware: ServiceBuilder<L>,
    _phantom: PhantomData<Log>,
}

impl<C, L, FutEffect, Log> ServerPrepare<C, L, FutEffect, Log> {
    fn new(prepares: SerialPrepareSet<C, FutEffect>, middleware: ServiceBuilder<L>) -> Self {
        Self {
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
        self.prepares.get_ref_configure().init_logger()?;
        Ok(ServerPrepare::new(self.prepares, self.middleware))
    }
}

impl<C: 'static> ServerPrepare<C, Identity, Ready<Result<EffectsCollector, PrepareError>>, NoLog> {
    pub fn with_config(config: C) -> Self
    where
        C: ServeAddress,
    {
        ServerPrepare::new(
            SerialPrepareSet::new(Arc::new(config)),
            ServiceBuilder::new(),
        )
    }
}

impl<C: 'static, L, FutEffect, Log> ServerPrepare<C, L, FutEffect, Log> {
    pub fn append_concurrent<F, Fut, R, G, E, S, Fr, Fg, Fe, Fs>(
        self,
        concurrent: F,
    ) -> ServerPrepare<
        C,
        L,
        impl Future<
            Output = Result<
                CombineEffects<Fr, Fg, Fe, Fs, EffectsCollector<R, G, E, S>>,
                PrepareError,
            >,
        >,
        Log,
    >
    where
        F: FnOnce(
                ConcurrentPrepareSet<C, Ready<Result<EffectsCollector, PrepareError>>>,
            ) -> ConcurrentPrepareSet<C, Fut>
            + 'static,
        Fut: Future<Output = Result<EffectsCollector<R, G, E, S>, PrepareError>>,
        FutEffect: Future<Output = Result<EffectsCollector<Fr, Fg, Fe, Fs>, PrepareError>>,
        Fr: RouteEffect,
        Fg: GracefulEffect,
        Fe: ExtensionEffect,
        Fs: ServerEffect,
        R: RouteEffect,
        G: GracefulEffect,
        E: ExtensionEffect,
        S: ServerEffect,
    {
        let concurrent_set = ConcurrentPrepareSet::new(self.prepares.get_configure());
        let prepares = concurrent(concurrent_set);
        let prepares = self.prepares.then_fut_effect(prepares.to_prepared_effect());
        ServerPrepare::new(prepares, self.middleware)
    }

    /// adding a [Prepare]
    ///
    /// ## Note
    ///
    /// the [Prepare] task will be waiting at the same time.
    ///
    /// **DO NOT** block any task for a long time, neither **sync** nor **async**
    pub fn append<P, R, S, G, E>(
        self,
        prepare: P,
    ) -> ServerPrepare<
        C,
        L,
        impl Future<Output = Result<CombineEffects<R, G, E, S, P::Effect>, PrepareError>>,
    >
    where
        FutEffect: Future<Output = Result<EffectsCollector<R, G, E, S>, PrepareError>>,
        R: RouteEffect,
        S: ServerEffect,
        G: GracefulEffect,
        E: ExtensionEffect,
        P: Prepare<C>,
    {
        let prepares = self.prepares.then(prepare);

        ServerPrepare::new(prepares, self.middleware)
    }
    /// adding a function-style [Prepare]
    pub fn append_fn<F, Args, R, S, G, E>(
        self,
        func: F,
    ) -> ServerPrepare<
        C,
        L,
        impl Future<
            Output = Result<
                CombineEffects<R, G, E, S, <FnPrepare<C, Args, F> as Prepare<C>>::Effect>,
                PrepareError,
            >,
        >,
    >
    where
        FutEffect: Future<Output = Result<EffectsCollector<R, G, E, S>, PrepareError>>,
        R: RouteEffect,
        S: ServerEffect,
        G: GracefulEffect,
        E: ExtensionEffect,
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
        ServerPrepare::new(self.prepares, self.middleware.layer(layer))
    }
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
    pub async fn prepare_start<Effect, NewResBody>(
        self,
    ) -> Result<ServerReady<impl IntoFuture<Output = Result<(), hyper::Error>>>, PrepareError>
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
        FutEffect: Future<Output = Result<Effect, PrepareError>>,
        Effect: PreparedEffect,
    {
        let (prepare_fut, configure) = self.prepares.unwrap();
        let (extension_effect, route_effect, graceful_effect, server_effect) =
            prepare_fut.await?.split_effect();

        let router = Router::new()
            // apply prepare effect on router
            .pipe(|router| route_effect.add_router(router))
            // apply prepare extension
            .pipe(|router| extension_effect.apply_extension(router))
            // adding middleware
            .pipe(|router| router.layer(self.middleware));

        let graceful = graceful_effect.set_graceful();

        let server = server::Server::bind(&ServeAddress::get_address(&*configure).into())
            // apply effect config server
            .pipe(|server| server_effect.config_serve(server))
            // apply configure config server
            .pipe(|server| configure.effect_server(server))
            .serve(router.into_make_service());

        Ok(match graceful {
            Some(fut) => ServerReady::Graceful(server.with_graceful_shutdown(fut)),
            None => ServerReady::Server(server),
        })
    }
}
