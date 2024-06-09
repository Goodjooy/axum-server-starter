use std::{convert::Infallible, sync::Arc};

use axum::{
    body::{Body, Bytes},
    handler::{Handler, HandlerService},
    response::Response,
    BoxError,
};
use http::Request;
use tap::Pipe;
use tokio::spawn;
use tower::{layer::util::Identity, util::MapResponseLayer, Layer, Service, ServiceBuilder};

use crate::{
    prepare_behave::effect_contain::TestRouter,
    prepare_sets::ContainerResult,
    server_prepare::{
        start_process::{
            graceful_shutdown::NoGraceful,
            logger::NoLog,
            state_ready::{StateNotReady, StateReady},
        },
        EmptyDecorator,
    },
    test_utils::TestResponse,
    FromStateCollector, PrepareStartError, SerialPrepareSet, ServerPrepare,
};

impl<C: 'static>
    ServerPrepare<
        C,
        ContainerResult<TestRouter, Identity>,
        NoLog,
        StateNotReady,
        NoGraceful,
        EmptyDecorator,
    >
{
    /// prepare staring the service with config
    pub fn test_with_config(config: C) -> Self {
        #[cfg(feature = "logger")]
        let span = tracing::debug_span!("prepare server start");
        #[cfg(not(feature = "logger"))]
        let span = crate::fake_span::FakeSpan;
        ServerPrepare::new(
            SerialPrepareSet::new_test(Arc::new(config), EmptyDecorator),
            NoGraceful,
            StateNotReady,
            span,
        )
    }
}

impl<C: 'static, Log, State, Graceful, L, Decorator>
    ServerPrepare<C, ContainerResult<TestRouter, L>, Log, StateReady<State>, Graceful, Decorator>
{
    /// prepare to a service for test
    ///
    /// this will consume `Self` then return a [Service](tower::Service) for the following test
    pub async fn preparing_test<NewResBody, H, T>(
        self,
        handler: H,
    ) -> Result<
        impl Service<Request<Body>, Response = TestResponse, Error = Infallible>,
        PrepareStartError,
    >
    where
        // middleware
        L: Send + 'static,
        L: Layer<HandlerService<H, T, State>>,
        L::Service: Service<Request<Body>, Response = Response<NewResBody>, Error = Infallible>
            + Send
            + Clone
            + 'static,
        NewResBody: http_body::Body<Data = Bytes> + Send + 'static,
        NewResBody::Error: Into<BoxError>,
        // state
        State: FromStateCollector,
        State: Clone + Send + 'static + Sync,
        // handler
        H: Handler<T, State>,
    {
        async {
            let (prepare_fut, _) = self.prepares.unwrap();
            debug!(execute = "Prepare");

            let (state, middleware, _) = prepare_fut.await?.unwrap();

            let state = State::fetch(state)?;

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

            let service = handler.with_state(state);
            Ok(ServiceBuilder::new()
                .layer(MapResponseLayer::new(TestResponse::new))
                .layer(middleware)
                .service(service))
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
