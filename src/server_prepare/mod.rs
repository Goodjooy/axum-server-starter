mod error;
use std::{
    any::type_name,
    convert::Infallible,
    error::Error,
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

use self::error::PrepareError;
pub use self::{
    pre_initial::{BoxPreparedEffect, Prepare, PreparedEffect},
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
    prepares: Vec<(&'static str, BoxPreparedEffect)>,

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
    pub fn append<P>(mut self, prepare: P) -> Self
    where
        P: Prepare<C>,
    {
        let task = prepare.prepare(Arc::clone(&self.config));
        self.prepares.push((type_name::<P>(), task));
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
        Box<dyn Error>,
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
        for (name, effect) in self.prepares {
            let mut effect = effect.await.map_err(|e| PrepareError::new(name, e))?;
            if let Some(fut) = effect.set_graceful() {
                graceful = Some(fut)
            }
            server_builder = effect.config_serve(server_builder);
            router = effect.add_router(router);

            let extension = ExtensionManage(router);
            let extension = effect.add_extension(extension);
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
