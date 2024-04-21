#[allow(unused_imports)]
use std::any::type_name;
use std::future::IntoFuture;
use std::{
    convert::Infallible,
    io,
    marker::{PhantomData, Send},
    sync::Arc,
};

use axum::body::Body;
use axum::{body::Bytes, routing::Route, BoxError, Router};
use futures::Future;
use hyper::{Request, Response};
use tap::Pipe;
use tokio::spawn;

use tower::{layer::util::Identity, Layer, Service, ServiceBuilder};

use crate::{
    prepare_behave::{effect_traits::PrepareRouteEffect, FromStateCollector},
    prepare_sets::ContainerResult,
    server_ready::ServerReady,
    SerialPrepareSet,
};

pub use self::{
    configure::{BindServe, EmptyDecorator, LoggerInitialization, PrepareDecorator, ServeAddress},
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
mod decorator;
mod post_prepare;

pub struct NoLog;

pub struct LogInit;

/// type for prepare starting
pub struct ServerPrepare<
    C,
    Effect,
    Log = LogInit,
    State = StateNotReady,
    Graceful = NoGraceful,
    Decorator = EmptyDecorator,
> {
    prepares: SerialPrepareSet<C, Effect, Decorator>,
    graceful: Graceful,
    state: State,
    #[cfg(feature = "logger")]
    span: tracing::Span,
    #[cfg(not(feature = "logger"))]
    span: crate::fake_span::FakeSpan,
    _phantom: PhantomData<(Log,)>,
}

impl<C, FutEffect, Log, State, Graceful, Decorator>
    ServerPrepare<C, FutEffect, Log, State, Graceful, Decorator>
{
    fn new(
        prepares: SerialPrepareSet<C, FutEffect, Decorator>,
        graceful: Graceful,
        state: State,
        #[cfg(feature = "logger")] span: tracing::Span,
        #[cfg(not(feature = "logger"))] span: crate::fake_span::FakeSpan,
    ) -> Self {
        Self {
            prepares,
            _phantom: PhantomData,
            state,
            span,
            graceful,
        }
    }
}

type LogResult<C, FutEffect, LogInit, State, Graceful, Decorator> = Result<
    ServerPrepare<C, FutEffect, LogInit, State, Graceful, Decorator>,
    <C as LoggerInitialization>::Error,
>;

impl<C, FutEffect, State, Graceful, Decorator>
    ServerPrepare<C, FutEffect, NoLog, State, Graceful, Decorator>
where
    C: LoggerInitialization,
{
    /// init the (logger) of this [ServerPrepare] ,require C impl [LoggerInitialization]
    pub fn init_logger(self) -> LogResult<C, FutEffect, LogInit, State, Graceful, Decorator> {
        self.span.in_scope(|| {
            let t = self.prepares.get_ref_configure().init_logger();
            info!(logger = "Init");
            t
        })?;

        Ok(ServerPrepare::new(
            self.prepares,
            self.graceful,
            self.state,
            self.span,
        ))
    }
}

impl<C: 'static>
    ServerPrepare<
        C,
        ContainerResult<(), Identity>,
        NoLog,
        StateNotReady,
        NoGraceful,
        EmptyDecorator,
    >
{
    /// prepare staring the service with config
    pub fn with_config(config: C) -> Self {
        #[cfg(feature = "logger")]
        let span = tracing::debug_span!("prepare server start");
        #[cfg(not(feature = "logger"))]
        let span = crate::fake_span::FakeSpan;
        ServerPrepare::new(
            SerialPrepareSet::new(Arc::new(config), EmptyDecorator),
            NoGraceful,
            StateNotReady,
            span,
        )
    }
}

impl<C: 'static, Log, State, Graceful, R, L, Decorator>
    ServerPrepare<C, ContainerResult<R, L>, Log, StateReady<State>, Graceful, Decorator>
{
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](ServerReady)
    pub async fn preparing<NewResBody>(
        self,
    ) -> Result<
        ServerReady<
            impl Future<Output = Result<(), io::Error>>,
            impl Future<Output = Result<(), io::Error>>,
        >,
        PrepareStartError,
    >
    where
        // config
        C: BindServe,
        // middleware
        L: Send + 'static,
        ServiceBuilder<L>: Layer<Route> + Clone,
        <ServiceBuilder<L> as Layer<Route>>::Service: Send
            + Clone
            + Service<Request<Body>, Response = Response<NewResBody>, Error = Infallible>
            + 'static,
        <<ServiceBuilder<L> as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
        NewResBody: http_body::Body<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
        // prepare task
        R: PrepareRouteEffect<State>,
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
                // apply to prepare effect on router
                .pipe(|router| route.set_route(router))
                // adding middleware
                .pipe(|router| router.layer(middleware))
                .with_state(state.clone());

            debug!(effect = "Graceful Shutdown");
            let graceful = self.graceful.get_graceful();

            debug!(effect = "Server");
            let listener = configure.bind().await?;
            let server = axum::serve(listener, router);
            debug!(effect = "All Done");
            info!(
                service.address = %&configure.get_address().into(),
                service.status = "Ready"
            );
            let post_prepare_tasks = self.state.take();
            debug!(
                execute = "Post Prepare Tasks",
                numbers = post_prepare_tasks.len()
            );
            for task in post_prepare_tasks {
                spawn({
                    let local_state = state.clone();
                    async move {
                        (task)(local_state).await;
                    }
                });
            }

            Ok(match graceful {
                Some(fut) => {
                    ServerReady::Graceful(server.with_graceful_shutdown(fut).into_future())
                }
                None => ServerReady::Server(server.into_future()),
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
