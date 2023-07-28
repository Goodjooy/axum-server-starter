use std::marker::PhantomData;

use crate::{prepare_behave::FromStateCollector, ServerPrepare};

pub struct StateNotReady;
pub struct StateReady<S>(PhantomData<S>);

impl<C, FutEffect, Log, Graceful,Decorator> ServerPrepare<C, FutEffect, Log, StateNotReady, Graceful,Decorator> {
    /// convert internal [`StateCollector`](crate::StateCollector) to special
    /// State
    pub fn convert_state<S: FromStateCollector>(
        self,
    ) -> ServerPrepare<C, FutEffect, Log, StateReady<S>, Graceful> {
        ServerPrepare {
            prepares: self.prepares,
            graceful: self.graceful,
            span: self.span,
            _phantom: PhantomData,
        }
    }
    /// convenient function for [`ServerPrepare::convert_state::<()>`](axum_starter::ServerPrepare::convert_state)
    pub fn no_state(self) -> ServerPrepare<C, FutEffect, Log, StateReady<()>, Graceful> {
        self.convert_state::<()>()
    }
}
