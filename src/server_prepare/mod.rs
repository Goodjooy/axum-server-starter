use std::{
    convert::Infallible,
    error::{self, Error},
    future::{Future, IntoFuture},
    marker::Send,
    pin::Pin,
    sync::Arc,
};

use axum::{
    body::Bytes,
    routing::{IntoMakeService, Route},
    Extension, Router,
};

use http_body::combinators::UnsyncBoxBody;
use hyper::{
    server::{self, conn::AddrIncoming},
    Body, Request, Response,
};
use tower::{
    layer::util::{Identity, Stack},
    Layer, Service, ServiceBuilder,
};

use crate::server_ready::ServerReady;

pub use self::{
    pre_initial::{InitialedEffect, PreEffect, PreInitial},
    serve_bind::{ServeBind, ServerEffect},
};
mod pre_initial;
mod serve_bind;

pub struct ExtensionManage(Router<Body>);

impl ExtensionManage {
    pub fn add_extension<S>(self, extension: S) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        Self(self.0.layer(Extension(extension)))
    }
}

/// type for prepare starting
pub struct ServerPrepare<C, L> {
    config: Arc<C>,
    prepares: Vec<InitialedEffect>,

    middleware: ServiceBuilder<L>,
}

impl<C> ServerPrepare<C, Identity> {
    pub fn with_config(config: C) -> Self
    where
        C: ServeBind,
    {
        ServerPrepare {
            config: Arc::new(config),
            prepares: Vec::new(),
            middleware: ServiceBuilder::new(),
        }
    }
}

impl<C, L> ServerPrepare<C, L> {
    pub fn append<P>(mut self, _: P) -> Self
    where
        P: PreInitial<Config = C>,
    {
        let task = P::init_this(Arc::clone(&self.config));
        self.prepares.push(task);
        self
    }

    pub fn with_global_middleware<M>(self, layer: M) -> ServerPrepare<C, Stack<M, L>> {
        ServerPrepare {
            middleware: self.middleware.layer(layer),
            config: self.config,
            prepares: self.prepares,
        }
    }

    pub async fn prepare_start(
        self,
    ) -> Result<
        ServerReady<
            AddrIncoming,
            IntoMakeService<Router<Body>>,
            impl IntoFuture<Output = Result<(), hyper::Error>>,
        >,
        Box<dyn error::Error>,
    >
    where
        C: ServeBind + ServerEffect,
        L: tower::Layer<axum::routing::Route>,
        <L as Layer<Route>>::Service: Send
            + Clone
            + Service<
                Request<Body>,
                Response = Response<UnsyncBoxBody<Bytes, Box<dyn Error + Sync + Send>>>,
                Error = Infallible,
            > + 'static,
        <<L as Layer<Route>>::Service as Service<Request<Body>>>::Future: Send,
    {
        let mut router = Router::new();
        let mut server_builder =
            server::Server::bind(&ServeBind::get_address(&*self.config).into());
        let mut graceful = Option::<Pin<Box<dyn Future<Output = ()>>>>::None;

        // apply all effect
        for effect in self.prepares {
            let mut effect = effect.await?;
            if let Some(fut) = effect.set_graceful() {
                graceful = Some(fut)
            }
            server_builder = effect.change_serve(server_builder);
            router = effect.adding_router(router);

            let extension = ExtensionManage(router);
            let extension = effect.adding_extract(extension);
            router = extension.0;
        }

        let router = router.layer(self.middleware);

        server_builder = self.config.effect_server(server_builder);

        let server = server_builder.serve(router.into_make_service());

        Ok(if let Some(fut) = graceful {
            ServerReady::Graceful(server.with_graceful_shutdown(fut))
        } else {
            ServerReady::Server(server)
        })
    }
}
