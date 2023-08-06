use std::future::Future;
use crate::{PrepareError, ServerPrepare};

impl<C, Effect, Log, State, Graceful, Decorator>
    ServerPrepare<C, Effect, Log, State, Graceful, Decorator>
{
    /// Add Decorator apply on every prepare [`futures::Future`]
    ///
    /// this will overwrite old [`PrepareDecorator`], combine multiply [`PrepareDecorator`] is **Not**
    ///support. Manual writing combining code instead
    pub fn set_decorator<D>(self, _: D) -> ServerPrepare<C, Effect, Log, State, Graceful, D>
    where
        D: PrepareDecorator,
    {
        ServerPrepare::new(self.prepares.change_decorator(), self.graceful, self.span)
    }
}

/// add decorator for each prepare 's [`Future`]
///
///It is useful for adding extra functional on original prepare task
pub trait PrepareDecorator: 'static {
    type OutFut<'fut, Fut, T>: Future<Output = Result<T, PrepareError>> + 'fut
        where
            Fut: Future<Output = Result<T, PrepareError>> + 'fut,
            T: 'static;

    fn decorator<'fut, Fut, T>(in_fut: Fut) -> Self::OutFut<'fut, Fut, T>
        where
            Fut: Future<Output = Result<T, PrepareError>> + 'fut,
            T: 'static;
}

/// Default Decorator without any effect
pub struct EmptyDecorator;

impl PrepareDecorator for EmptyDecorator {
    type OutFut<'fut, Fut, T> = Fut
        where Fut: Future<Output=Result<T, PrepareError>> + 'fut  ,T: 'static;

    fn decorator<'fut, Fut, T>(in_fut: Fut) -> Self::OutFut<'fut, Fut, T>
        where
            Fut: Future<Output = Result<T, PrepareError>> + 'fut,
            T: 'static,
    {
        in_fut
    }
}
