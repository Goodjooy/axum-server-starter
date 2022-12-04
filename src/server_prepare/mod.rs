mod adding_middleware;
mod adding_prepare;
mod error;
mod graceful_shutdown;
mod state_ready;
#[allow(unused_imports)]
use std::any::type_name;
use std::{
    convert::Infallible,
    marker::{PhantomData, Send},
    sync::Arc,
};

use axum::{body::Bytes, routing::Route, BoxError, Router};

use futures::{future::Ready, Future};

use hyper::{server, Body, Request, Response};
use tap::Pipe;
use tower::{layer::util::Identity, Layer, Service, ServiceBuilder};

use crate::{
    prepare_behave::{effect_traits::PrepareRouteEffect, EffectContainer, FromStateCollector},
    server_ready::ServerReady,
    SerialPrepareSet,
};

pub use self::{
    configure::{ConfigureServerEffect, LoggerInitialization, ServeAddress},
    error::{PrepareError, PrepareStartError},
};
use self::{
    graceful_shutdown::{FetchGraceful, NoGraceful},
    state_ready::{StateNotReady, StateReady},
};
mod configure;

pub struct NoLog;
pub struct LogInit;

/// type for prepare starting
pub struct ServerPrepare<C, FutEffect, Log = LogInit, State = StateNotReady, Graceful = NoGraceful>
{
    prepares: SerialPrepareSet<C, FutEffect>,
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

impl<C: 'static>
    ServerPrepare<
        C,
        Ready<Result<EffectContainer<(), Identity>, PrepareError>>,
        NoLog,
        StateNotReady,
        NoGraceful,
    >
{
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
impl<C: 'static, FutEffect, Log, State, Graceful>
    ServerPrepare<C, FutEffect, Log, StateReady<State>, Graceful>
{
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
    pub async fn prepare_start<R, LayerInner, NewResBody>(
        self,
    ) -> Result<
        ServerReady<
            impl Future<Output = Result<(), hyper::Error>>,
            impl Future<Output = Result<(), hyper::Error>>,
        >,
        PrepareStartError,
    >
    where
        // config
        C: ServeAddress + ConfigureServerEffect,
        // middleware
        LayerInner: Send + 'static,
        ServiceBuilder<LayerInner>: Layer<Route> + Clone,
        <ServiceBuilder<LayerInner> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response = Response<NewResBody>, Error = Infallible>
            + 'static,
        <<ServiceBuilder<LayerInner> as Layer<Route>>::Service as Service<Request<Body>>>::Future:
            Send,
        NewResBody: http_body::Body<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
        // prepare task
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareError>>,
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
            let server = server::Server::bind(&ServeAddress::get_address(&*configure).into())
                // apply configure config server
                .pipe(|server| configure.effect_server(server))
                .serve(router.into_make_service());

            debug!(effect = "All Done");
            info!(
                service.address = %configure.get_address().into(),
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
