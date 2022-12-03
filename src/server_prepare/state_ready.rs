use std::marker::PhantomData;

use crate::{prepare_behave::FromStateCollector, ServerPrepare};

pub struct StateNotReady;
pub struct StateReady<S>(PhantomData<S>);

impl<C, FutEffect, Log, Graceful> ServerPrepare<C, FutEffect, Log, StateNotReady, Graceful> {
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
}
