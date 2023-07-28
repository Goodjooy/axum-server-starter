use std::{
    convert::Infallible,
    marker::{PhantomData, Send},
    sync::Arc,
};
#[allow(unused_imports)]
use std::any::type_name;

use axum::{body::Bytes, BoxError, Router, routing::Route};
use futures::Future;
use hyper::{Body, Request, Response};
use hyper::server::accept::Accept;
use tap::Pipe;
use tokio::io::{AsyncRead, AsyncWrite};
use tower::{Layer, layer::util::Identity, Service, ServiceBuilder};

use crate::{
    prepare_behave::{effect_traits::PrepareRouteEffect, FromStateCollector},
    prepare_sets::ContainerResult,
    SerialPrepareSet,
    server_ready::ServerReady,
};

pub use self::{
    configure::{BindServe, ConfigureServerEffect, LoggerInitialization, ServeAddress},
    error::{PrepareError, PrepareStartError},
};
use self::{
    graceful_shutdown::{FetchGraceful, NoGraceful},
    state_ready::{StateNotReady, StateReady},
};

mod adding_middleware;
mod adding_prepare;
mod error;
mod graceful_shutdown;
mod state_ready;


mod configure;

pub struct NoLog;

pub struct LogInit;

/// type for prepare starting
pub struct ServerPrepare<C, Effect, Log = LogInit, State = StateNotReady, Graceful = NoGraceful> {
    prepares: SerialPrepareSet<C, Effect>,
    graceful: Graceful,
    #[cfg(feature = "logger")]
    span: tracing::Span,
    #[cfg(not(feature = "logger"))]
    span: crate::fake_span::FakeSpan,
    _phantom: PhantomData<(Log, State)>,
}

impl<C, FutEffect, Log, State, Graceful> ServerPrepare<C, FutEffect, Log, State, Graceful> {
    fn new(
        prepares: SerialPrepareSet<C, FutEffect>,
        graceful: Graceful,
        #[cfg(feature = "logger")] span: tracing::Span,
        #[cfg(not(feature = "logger"))] span: crate::fake_span::FakeSpan,
    ) -> Self {
        Self {
            prepares,
            _phantom: PhantomData,
            span,
            graceful,
        }
    }
}

impl<C, FutEffect, State, Graceful> ServerPrepare<C, FutEffect, NoLog, State, Graceful>
    where
        C: LoggerInitialization,
{
    /// init the logger of this [ServerPrepare] ,require C impl [LoggerInitialization]
    pub fn init_logger(
        self,
    ) -> Result<ServerPrepare<C, FutEffect, LogInit, State, Graceful>, C::Error> {
        self.span.in_scope(|| {
            let t = self.prepares.get_ref_configure().init_logger();
            info!(logger = "Init");
            t
        })?;

        Ok(ServerPrepare::new(self.prepares, self.graceful, self.span))
    }
}

impl<C: 'static> ServerPrepare<C, ContainerResult<(), Identity>, NoLog, StateNotReady, NoGraceful> {
    /// prepare staring the service with config
    pub fn with_config(config: C) -> Self
        where
            C: ServeAddress,
    {
        #[cfg(feature = "logger")]
            let span = tracing::debug_span!("prepare server start");
        #[cfg(not(feature = "logger"))]
            let span = crate::fake_span::FakeSpan;
        ServerPrepare::new(SerialPrepareSet::new(Arc::new(config)), NoGraceful, span)
    }
}

impl<C: 'static, Log, State, Graceful, R, L>
ServerPrepare<C, ContainerResult<R, L>, Log, StateReady<State>, Graceful>
{
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
    #[deprecated]
    pub async fn prepare_start<NewResBody>(
        self,
    ) -> Result<
        ServerReady<
            impl Future<Output=Result<(), hyper::Error>>,
            impl Future<Output=Result<(), hyper::Error>>,
        >,
        PrepareStartError,
    >
        where
        // config
            C: BindServe + ConfigureServerEffect<<C as BindServe>::A>,
            <C::A as Accept>::Conn: AsyncRead + AsyncWrite + Send + Sync + Unpin,
            <C::A as Accept>::Error: Send + Sync + std::error::Error,
        // middleware
            L: Send + 'static,
            ServiceBuilder<L>: Layer<Route> + Clone,
            <ServiceBuilder<L> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response=Response<NewResBody>, Error=Infallible>
            + 'static,
            <<ServiceBuilder<L> as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
            NewResBody: http_body::Body<Data=Bytes> + Send + 'static,
            NewResBody::Error: Into<BoxError>,
        // prepare task
            R: PrepareRouteEffect<State, Body>,
        // state
            State: FromStateCollector,
            State: Clone + Send + 'static + Sync,
        // graceful
            Graceful: FetchGraceful, {
        self.preparing().await
    }
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
    pub async fn preparing<NewResBody>(
        self,
    ) -> Result<
        ServerReady<
            impl Future<Output=Result<(), hyper::Error>>,
            impl Future<Output=Result<(), hyper::Error>>,
        >,
        PrepareStartError,
    >
        where
        // config
            C: BindServe + ConfigureServerEffect<<C as BindServe>::A>,
            <C::A as Accept>::Conn: AsyncRead + AsyncWrite + Send + Sync + Unpin,
            <C::A as Accept>::Error: Send + Sync + std::error::Error,
        // middleware
            L: Send + 'static,
            ServiceBuilder<L>: Layer<Route> + Clone,
            <ServiceBuilder<L> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response=Response<NewResBody>, Error=Infallible>
            + 'static,
            <<ServiceBuilder<L> as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
            NewResBody: http_body::Body<Data=Bytes> + Send + 'static,
            NewResBody::Error: Into<BoxError>,
        // prepare task
            R: PrepareRouteEffect<State, Body>,
        // state
            State: FromStateCollector,
            State: Clone + Send + 'static + Sync,
        // graceful
            Graceful: FetchGraceful,
    {
        async {
            let (prepare_fut, configure) = self.prepares.unwrap();
            debug!(execute = "Prepare");

            let (state, middleware, route) = prepare_fut.await?.unwrap();

            let state = State::fetch(state)?;

            debug!(effect = "Router");
            let router = Router::new()
                // apply prepare effect on router
                .pipe(|router| route.set_route(router))
                // adding middleware
                .pipe(|router| router.layer(middleware))
                .with_state(state);

            debug!(effect = "Graceful Shutdown");
            let graceful = self.graceful.get_graceful();

            debug!(effect = "Server");
            let server = configure.bind()
                // apply configure config server
                .pipe(|server| configure.effect_server(server))
                .serve(router.into_make_service());

            debug!(effect = "All Done");
            info!(
                service.address = %&configure.listen_target(),
                service.status = "Ready"
            );

            Ok(match graceful {
                Some(fut) => ServerReady::Graceful(server.with_graceful_shutdown(fut)),
                None => ServerReady::Server(server),
            })
        }
            .pipe(|fut| {
                #[cfg(feature = "logger")]
                {
                    tracing::Instrument::instrument(fut, self.span)
                }
                #[cfg(not(feature = "logger"))]
                {
                    fut
                }
            })
            .await
    }
}
