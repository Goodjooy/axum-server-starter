pub mod extension;
pub mod graceful;
pub mod router;
pub mod service;
use std::{future::Future, sync::Arc};

use self::{
    extension::ExtensionEffect, graceful::GracefulEffect, router::RouteEffect,
    service::ServerEffect,
};

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
    fn split_effect(
        self,
    ) -> EffectGroup<Self::Route, Self::Extension, Self::Graceful, Self::Server>;
}

pub struct EffectGroup<R, E, G, S> {
    pub extension: E,
    pub route: R,
    pub graceful: G,
    pub service: S,
}
