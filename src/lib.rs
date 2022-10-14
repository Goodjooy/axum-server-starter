#![forbid(unsafe_code)] 

mod config_provide;
mod effect_utils;
mod prepare_handler;
mod prepared_effect;
mod server_prepare;
mod server_ready;
pub use server_prepare::{
    BoxPreparedEffect, ExtensionManage, Prepare, PreparedEffect, ServeAddress, ServerEffect,
    ServerPrepare,
};
pub use server_ready::ServerReady;

pub use config_provide::{from_config::FromConfig, provider::Provider};
pub use prepare_handler::{fn_prepare, FnPrepare, PrepareHandler};

pub use axum_starter_macro::{prepare, Provider};
pub use prepared_effect::IntoFallibleEffect;

pub use effect_utils::{extension, graceful, router, service};
