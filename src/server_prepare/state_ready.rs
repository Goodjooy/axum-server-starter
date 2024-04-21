use std::marker::PhantomData;

use crate::{prepare_behave::FromStateCollector, ServerPrepare};
use crate::server_prepare::post_prepare::PostPrepareFn;

pub struct StateNotReady;

pub struct StateReady<S>(Vec<PostPrepareFn<S>>);

impl<S> Default for StateReady<S> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<S> StateReady<S> {
    pub fn push(&mut self, post_prepare:PostPrepareFn<S>){
        self.0.push(post_prepare)
    }
    
    pub fn take(self)->Vec<PostPrepareFn<S>>{
        self.0
    }
}

impl<C, FutEffect, Log, Graceful, Decorator>
    ServerPrepare<C, FutEffect, Log, StateNotReady, Graceful, Decorator>
{
    /// convert internal [`StateCollector`](crate::StateCollector) to special
    /// State
    pub fn convert_state<S: FromStateCollector>(
        self,
    ) -> ServerPrepare<C, FutEffect, Log, StateReady<S>, Graceful, Decorator> {
        ServerPrepare {
            prepares: self.prepares,
            graceful: self.graceful,
            state:StateReady::default(),
            span: self.span,
            _phantom: PhantomData,
        }
    }
    /// convenient function for [`ServerPrepare::convert_state::<()>`](axum_starter::ServerPrepare::convert_state)
    pub fn no_state(self) -> ServerPrepare<C, FutEffect, Log, StateReady<()>, Graceful, Decorator> {
        self.convert_state::<()>()
    }
}
