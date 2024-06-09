#[allow(unused_imports)]
use std::any::type_name;
use std::marker::PhantomData;

use crate::SerialPrepareSet;

pub use self::error::{PrepareError, PrepareStartError};
pub use self::start_process::configure::{
    BindServe, EmptyDecorator, LoggerInitialization, PrepareDecorator, ServeAddress,
};
use self::start_process::{
    graceful_shutdown::NoGraceful, logger::LogInit, state_ready::StateNotReady,
};

mod error;
mod prepare;
mod start_process;

/// type for prepare starting
pub struct ServerPrepare<
    C,
    Effect,
    Log = LogInit,
    State = StateNotReady,
    Graceful = NoGraceful,
    Decorator = EmptyDecorator,
> {
    prepares: SerialPrepareSet<C, Effect, Decorator>,
    graceful: Graceful,
    state: State,
    #[cfg(feature = "logger")]
    span: tracing::Span,
    #[cfg(not(feature = "logger"))]
    span: crate::fake_span::FakeSpan,
    _phantom: PhantomData<(Log,)>,
}

impl<C, FutEffect, Log, State, Graceful, Decorator>
    ServerPrepare<C, FutEffect, Log, State, Graceful, Decorator>
{
    fn new(
        prepares: SerialPrepareSet<C, FutEffect, Decorator>,
        graceful: Graceful,
        state: State,
        #[cfg(feature = "logger")] span: tracing::Span,
        #[cfg(not(feature = "logger"))] span: crate::fake_span::FakeSpan,
    ) -> Self {
        Self {
            prepares,
            _phantom: PhantomData,
            state,
            span,
            graceful,
        }
    }
}
