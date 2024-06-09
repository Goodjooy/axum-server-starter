use std::{convert::Infallible, future::IntoFuture, io, sync::Arc};

use axum::{
    body::{Body, Bytes},
    routing::Route,
    BoxError, Router,
};
use futures::Future;
use hyper::{Request, Response};
use tap::Pipe;
use tokio::spawn;
use tower::{layer::util::Identity, Layer, Service, ServiceBuilder};

use crate::{
    prepare_behave::effect_contain::BaseRouter,
    prepare_sets::ContainerResult,
    server_prepare::{
        start_process::{
            graceful_shutdown::{FetchGraceful, NoGraceful},
            logger::NoLog,
            state_ready::{StateNotReady, StateReady},
        },
        EmptyDecorator,
    },
    BindServe, FromStateCollector, PrepareRouteEffect, PrepareStartError, SerialPrepareSet,
    ServerPrepare, ServerReady,
};

impl<C: 'static>
    ServerPrepare<
        C,
        ContainerResult<BaseRouter<()>, Identity>,
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
    ServerPrepare<C, ContainerResult<BaseRouter<R>, L>, Log, StateReady<State>, Graceful, Decorator>
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

            let (state, middleware, BaseRouter(route)) = prepare_fut.await?.unwrap();

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
