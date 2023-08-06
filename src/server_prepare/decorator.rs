use crate::{Prepare, PrepareError, ServerPrepare};
use std::any::type_name;
use std::future::Future;

impl<C, Effect, Log, State, Graceful, Decorator>
    ServerPrepare<C, Effect, Log, State, Graceful, Decorator>
{
    /// Add Decorator apply on every prepare [`futures::Future`]
    ///
    /// this will overwrite old [`PrepareDecorator`], combine multiply [`PrepareDecorator`] is **Not**
    ///support. Manual writing combining code instead
    pub fn set_decorator<D>(self, decorator: D) -> ServerPrepare<C, Effect, Log, State, Graceful, D>
    where
        D: PrepareDecorator,
    {
        ServerPrepare::new(
            self.prepares.change_decorator(decorator),
            self.graceful,
            self.span,
        )
    }
}

/// add decorator for each prepare 's [`Future`]
///
///It is useful for adding extra functional on original prepare task
pub trait PrepareDecorator: 'static {
    type OutFut<Fut, T>: Future<Output = Result<T, PrepareError>> + 'static
    where
        Fut: Future<Output = Result<T, PrepareError>> + 'static,
        T: 'static;

    fn decorator<Fut, T>(&self, src: &'static str, in_fut: Fut) -> Self::OutFut<Fut, T>
    where
        Fut: Future<Output = Result<T, PrepareError>> + 'static,
        T: 'static;

    fn prepare_decorator<C, P, Fut>(&self, in_fut: Fut) -> Self::OutFut<Fut, P::Effect>
    where
        Fut: Future<Output = Result<P::Effect, PrepareError>> + 'static,
        P: Prepare<C>,
        C: 'static,
    {
        PrepareDecorator::decorator(self, type_name::<P>(), in_fut)
    }
}

/// Default Decorator without any effect
pub struct EmptyDecorator;

impl PrepareDecorator for EmptyDecorator {
    type OutFut<Fut, T> = Fut where Fut: Future<Output=Result<T, PrepareError>> + 'static, T: 'static;

    fn decorator<Fut, T>(&self, _src: &'static str, in_fut: Fut) -> Self::OutFut<Fut, T>
    where
        Fut: Future<Output = Result<T, PrepareError>> + 'static,
        T: 'static,
    {
        in_fut
    }
}
