mod error;
use std::{
    any::type_name, convert::Infallible, error::Error, future::IntoFuture, marker::Send, sync::Arc,
};

use axum::{
    body::Bytes,
    routing::{IntoMakeService, Route},
    Router,
};

use futures::{stream::iter, StreamExt, TryFutureExt, TryStreamExt};
use http_body::combinators::UnsyncBoxBody;
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

use self::error::PrepareError;
pub use self::{
    prepare::{BoxPreparedEffect, ExtensionManage, Prepare, PreparedEffect},
    serve_bind::{ServeAddress, ServerEffect},
};
mod prepare;
mod serve_bind;

/// type for prepare starting
pub struct ServerPrepare<C, L> {
    config: Arc<C>,
    prepares: Vec<(&'static str, BoxPreparedEffect)>,

    middleware: ServiceBuilder<L>,
}

impl<C> ServerPrepare<C, Identity> {
    pub fn with_config(config: C) -> Self
    where
        C: ServeAddress,
    {
        ServerPrepare {
            config: Arc::new(config),
            prepares: Vec::new(),
            middleware: ServiceBuilder::new(),
        }
    }
}

impl<C, L> ServerPrepare<C, L> {
    /// adding a [Prepare]
    pub fn append<P>(mut self, prepare: P) -> Self
    where
        P: Prepare<C>,
    {
        let task = prepare.prepare(Arc::clone(&self.config));
        self.prepares.push((type_name::<P>(), task));
        self
    }
    /// adding a function-style [Prepare]
    pub fn append_fn<F, Args>(self, func: F) -> Self
    where
        F: PrepareHandler<Args, C>,
    {
        self.append(fn_prepare(func))
    }
    /// adding global middleware
    ///
    /// ## note
    /// before call [Self::prepare_start] make sure the [Service::Response] is meet the
    /// axum requirement
    pub fn with_global_middleware<M>(self, layer: M) -> ServerPrepare<C, Stack<M, L>> {
        ServerPrepare {
            middleware: self.middleware.layer(layer),
            config: self.config,
            prepares: self.prepares,
        }
    }
    /// prepare to start this server
    ///
    /// this will consume `Self` then return [ServerReady](crate::ServerReady)
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
        C: ServeAddress + ServerEffect,
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
        let mut effects = iter(self.prepares)
            .then(|(name, fut)| fut.map_err(move |err| PrepareError::new(name, err)))
            .try_collect::<Vec<_>>()
            .await?;

        let router = Router::new()
            // apply prepare effect on router
            .pipe(|router| {
                effects
                    .iter_mut()
                    .fold(router, |router, effect| effect.add_router(router))
            })
            // apply prepare extension
            .pipe(|router| {
                effects
                    .iter_mut()
                    .fold(router, |router, effect| effect.apply_extension(router))
            })
            // adding middleware
            .pipe(|router| router.layer(self.middleware));

        let graceful = effects.iter_mut().fold(None, |graceful, effect| {
            match (graceful, effect.set_graceful()) {
                (None, grace @ Some(_)) | (grace @ Some(_), _) => grace,
                (None, None) => None,
            }
        });

        let server = server::Server::bind(&ServeAddress::get_address(&*self.config).into())
            // apply effect config server
            .pipe(|server| {
                effects
                    .iter_mut()
                    .fold(server, |server, effect| effect.config_serve(server))
            })
            // apply configure config server
            .pipe(|server| self.config.effect_server(server))
            .serve(router.into_make_service());

        Ok(match graceful {
            Some(fut) => ServerReady::Graceful(server.with_graceful_shutdown(fut)),
            None => ServerReady::Server(server),
        })
    }
}
