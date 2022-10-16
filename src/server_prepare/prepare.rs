use std::{future::Future, sync::Arc};

use axum::{Extension, Router};
use hyper::{
    server::{self, conn::AddrIncoming},
    Body,
};
use tap::Pipe;

/// Preparing before server launch
///
/// the generic argument `Config` represent to the server config object
pub trait Prepare<Config: 'static> {
    type Effect: PreparedEffect;
    type Error: std::error::Error + 'static;
    type Future: Future<Output = Result<Self::Effect, Self::Error>>;
    /// preparing before sever start, return the effect after this preparing finish
    ///
    /// - if there is not any problem during preparing, return `Ok()`
    /// - otherwise, return `Err()`
    fn prepare(self, config: Arc<Config>) -> Self::Future;
}

/// side effect after a [Prepare]
pub trait PreparedEffect {
    type Extension: ExtensionEffect;
    type Graceful: GracefulEffect;
    type Route: RouteEffect;
    type Server: ServerEffect;

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server);
}

pub trait ExtensionEffect: Sized {
    fn apply_extension(self, router: Router) -> Router {
        router
            .pipe(ExtensionManage)
            .pipe(|extension| self.add_extension(extension))
            .pipe(|ExtensionManage(router)| router)
    }

    /// [Prepare] want to add `Extension` s
    fn add_extension(self, extension: ExtensionManage) -> ExtensionManage {
        extension
    }
}

pub trait GracefulEffect: Sized {
    type GracefulFuture: Future<Output = ()>;
    /// [Prepare] want to set a graceful shutdown signal returning `[Option::Some]`
    ///
    /// ## Warning
    /// if there are multiply [Prepare] want to set graceful shutdown, the first one set the signal will be applied
    fn set_graceful(self) -> Option<Self::GracefulFuture>;
}

pub trait RouteEffect: Sized {
    /// prepare want to adding routing on the root router
    ///
    /// ## Note
    /// [PreparedEffect::add_extension] will be applied after [PreparedEffect::add_router] being applied.
    ///
    /// the router adding by a [PrepareEffect](crate::PreparedEffect) can safely using Extension adding by
    /// [PreparedEffect::add_extension] in the same [PrepareEffect](crate::PreparedEffect)
    fn add_router(self, router: Router) -> Router {
        router
    }
}

pub trait ServerEffect: Sized {
    /// changing the serve config
    ///
    /// ## Note
    /// the changing of server config might be overwrite by `Config` using [crate::ServerEffect]
    fn config_serve(self, server: server::Builder<AddrIncoming>) -> server::Builder<AddrIncoming> {
        server
    }
}

/// A help type for adding extension
pub struct ExtensionManage(pub(crate) Router<Body>);

impl ExtensionManage {
    pub fn add_extension<S>(self, extension: S) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        Self(self.0.layer(Extension(extension)))
    }
}
