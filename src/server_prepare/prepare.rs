use std::{error::Error, future::Future, pin::Pin, sync::Arc};

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
    /// preparing before sever start, return the effect after this preparing finish
    ///
    /// return type is `Pin<Box<dyn Future<Output = Result<Box<dyn PreparedEffect>,Box<dyn Error>>>>>`
    /// - if there is not any problem during preparing, return `Ok()`
    /// - otherwise, return `Err()`
    fn prepare(self, config: Arc<Config>) -> BoxPreparedEffect;
}

/// boxed [Future] which return a [Result], with boxed [PreparedEffect] and [Error]
pub type BoxPreparedEffect =
    Pin<Box<dyn Future<Output = Result<Box<dyn PreparedEffect>, Box<dyn Error>>>>>;

/// side effect after a [Prepare]
pub trait PreparedEffect {
    fn apply_extension(&mut self, router: Router) -> Router {
        router
            .pipe(ExtensionManage)
            .pipe(|extension| self.add_extension(extension))
            .pipe(|ExtensionManage(router)| router)
    }

    /// [Prepare] want to add `Extension` s
    fn add_extension(&mut self, extension: ExtensionManage) -> ExtensionManage {
        extension
    }
    /// [Prepare] want to set a graceful shutdown signal returning `[Option::Some]`
    ///
    /// ## Warning
    /// if there are multiply [Prepare] want to set graceful shutdown, the first one set the signal will be applied
    fn set_graceful(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        None
    }

    /// changing the serve config
    ///
    /// ## Note
    /// the changing of server config might be overwrite by `Config` using [crate::ServerEffect]
    fn config_serve(&self, server: server::Builder<AddrIncoming>) -> server::Builder<AddrIncoming> {
        server
    }

    /// prepare want to adding routing on the root router
    ///
    /// ## Note
    /// [PreparedEffect::add_extension] will be applied after [PreparedEffect::add_router] being applied.
    ///
    /// the router adding by a [PrepareEffect](crate::PreparedEffect) can safely using Extension adding by
    /// [PreparedEffect::add_extension] in the same [PrepareEffect](crate::PreparedEffect)
    fn add_router(&mut self, router: Router) -> Router {
        router
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
