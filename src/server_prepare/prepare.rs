use std::{future::Future, pin::Pin, sync::Arc};

use axum::{Extension, Router};
use hyper::{
    server::{conn::AddrIncoming, Builder},
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
    /// the effect on extension
    type Extension: ExtensionEffect;
    /// the effect on setting graceful shutdown signal
    type Graceful: GracefulEffect;
    /// the effect on changing **root** [Router]
    type Route: RouteEffect;
    /// the effect on changing [Server](Builder)
    type Server: ServerEffect;

    /// split this [PreparedEffect] into 4 different part
    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server);
}

/// [Prepare] effect may adding `[Extension]`s
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

/// [Prepare] effect may set a graceful shutdown signal
pub trait GracefulEffect: Sized {
    /// [Prepare] want to set a graceful shutdown signal returning `[Option::Some]`
    ///
    /// ## Warning
    /// if there are multiply [Prepare] want to set graceful shutdown, the first one set the signal will be applied
    fn set_graceful(self) -> Option<Pin<Box<dyn Future<Output = ()>>>>;
}

/// [Prepare] effect may apply [Router::route], [Router::nest], [Router::fallback], [Router::merge] on the root [Router]
pub trait RouteEffect: Sized {
    /// prepare want to adding routing on the root router
    ///
    /// ## Note
    /// [ExtensionEffect::add_extension] will be applied after [RouteEffect::add_router] being applied.
    ///
    /// the router adding by a [RouteEffect] can safely using Extension adding by
    /// [ExtensionEffect::add_extension] in the same [PrepareEffect](crate::PreparedEffect)
    fn add_router(self, router: Router) -> Router {
        router
    }
}

/// [Prepare] effect may change the server configure
pub trait ServerEffect: Sized {
    /// changing the serve config
    ///
    /// ## Note
    /// the changing of server config might be overwrite by `Config` using [crate::ServerEffect]
    fn config_serve(self, server: Builder<AddrIncoming>) -> Builder<AddrIncoming> {
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
