use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use axum::Router;
use hyper::server::{self, conn::AddrIncoming};

use super::ExtensionManage;

pub trait Prepare<Config> {

    fn prepare(self, config: Arc<Config>) -> BoxPreparedEffect;
}

pub type BoxPreparedEffect =
    Pin<Box<dyn Future<Output = Result<Box<dyn PreparedEffect>, Box<dyn Error>>>>>;

pub trait PreparedEffect {
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
