#![forbid(unsafe_code)]
#![doc = include_str!("../Readme.md")]
#[macro_use]
mod log_macro;

mod config_provide;
mod effect_utils;
mod fake_span;
mod prepare_behave;
mod prepare_sets;
mod server_prepare;
mod server_ready;

#[cfg(feature = "test-utils")]
mod test_utils;

pub use prepare_behave::effect_collectors::state_collector::{
    FromStateCollector, StateCollector, TypeNotInState,
};
pub use prepare_behave::effect_traits::{
    Prepare, PrepareMiddlewareEffect, PrepareRouteEffect, PrepareStateEffect,
};
pub use server_prepare::{
    BindServe, LoggerInitialization, PrepareDecorator, PrepareError, PrepareStartError,
    ServeAddress, ServerPrepare,
};
pub use server_ready::ServerReady;

pub use axum_starter_macro::{prepare, Configure, FromStateCollector, Provider};
pub use config_provide::provider::Provider;
pub use effect_utils::{router, state};
pub use futures::future::{ready, Ready};
pub use prepare_sets::{concurrent_set::ConcurrentPrepareSet, serial_set::SerialPrepareSet};
#[cfg(feature = "test-utils")]
pub use test_utils::TestResponse;
