#![forbid(unsafe_code)]
#![doc = include_str!("../Readme.md")]
mod config_provide;
mod effect_utils;
mod log_macro;
mod prepare_handler;
mod prepare_sets;
mod prepared_effect;
mod server_prepare;
mod server_ready;
pub use server_prepare::{
    ConfigureServerEffect, ExtensionEffect, ExtensionManage, GracefulEffect, LoggerInitialization,
    Prepare, PrepareError, PreparedEffect, RouteEffect, ServeAddress, ServerEffect, ServerPrepare,
};
pub use server_ready::ServerReady;

pub use config_provide::provider::Provider;
pub use prepare_handler::{fn_prepare, FnPrepare, PrepareHandler};

pub use axum_starter_macro::{prepare, Configure, Provider};
pub use prepared_effect::{EffectsCollector, IntoFallibleEffect};

pub use effect_utils::{extension, graceful, router, service};
pub use prepare_sets::{concurrent_set::ConcurrentPrepareSet, serial_set::SerialPrepareSet};

pub use hyper::server::{conn::AddrIncoming, Builder};
