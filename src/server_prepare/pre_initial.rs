use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use axum::{Extension, Router};
use hyper::{
    server::{self, conn::AddrIncoming},
    Body,
};
use tap::Pipe;

pub trait Prepare<Config> {
    fn prepare(self, config: Arc<Config>) -> BoxPreparedEffect;
}

pub type BoxPreparedEffect =
    Pin<Box<dyn Future<Output = Result<Box<dyn PreparedEffect>, Box<dyn Error>>>>>;

pub trait PreparedEffect {
    fn apply_extension(&mut self, router: Router) -> Router {
        router
            .pipe(ExtensionManage)
            .pipe(|extension| self.add_extension(extension))
            .pipe(|ExtensionManage(router)| router)
    }

    fn add_extension(&mut self, extension: ExtensionManage) -> ExtensionManage {
        extension
    }

    fn set_graceful(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        None
    }

    fn config_serve(&self, server: server::Builder<AddrIncoming>) -> server::Builder<AddrIncoming> {
        server
    }

    fn add_router(&mut self, router: Router) -> Router {
        router
    }
}

impl PreparedEffect for () {}
pub struct ExtensionManage(pub(crate) Router<Body>);

impl ExtensionManage {
    pub fn add_extension<S>(self, extension: S) -> Self
    where
        S: Clone + Send + Sync + 'static,
    {
        Self(self.0.layer(Extension(extension)))
    }
}
