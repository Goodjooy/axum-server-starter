use std::{error::Error, future::Future, pin::Pin, sync::Arc};

use axum::Router;
use hyper::server::{self, conn::AddrIncoming};

use super::ExtensionManage;

pub trait PreInitial {
    type Config;

    fn init_this<'r>(config: Arc<Self::Config>) -> InitialedEffect;
}

pub type InitialedEffect =
    Pin<Box<dyn Future<Output = Result<Box<dyn PreEffect>, Box<dyn Error>>>>>;

pub trait PreEffect {
    fn adding_extract(&mut self, extension: ExtensionManage) -> ExtensionManage {
        extension
    }

    fn set_graceful(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        None
    }

    fn change_serve(&self, server: server::Builder<AddrIncoming>) -> server::Builder<AddrIncoming> {
        server
    }

    fn adding_router(&mut self, router: Router) -> Router {
        router
    }
}
