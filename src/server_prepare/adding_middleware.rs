use futures::Future;
use tower::layer::util::Stack;

use crate::{prepare_behave::EffectContainer, PrepareStartError, ServerPrepare};

impl<C: 'static, FutEffect, Log, State, Graceful>
    ServerPrepare<C, FutEffect, Log, State, Graceful>
{
    /// adding middleware without previously [Prepare](crate::Prepare) action
    pub fn layer<R, LayerInner, M>(
        self,
        middleware: M,
    ) -> ServerPrepare<
        C,
        impl Future<Output = Result<EffectContainer<R, Stack<M, LayerInner>>, PrepareStartError>>,
        Log,
        State,
        Graceful,
    >
    where
        FutEffect: Future<Output = Result<EffectContainer<R, LayerInner>, PrepareStartError>>,
    {
        self.span.in_scope(|| {
            debug!(middleware.layer = core::any::type_name::<M>());
        });
        ServerPrepare::new(
            self.prepares.set_middleware(middleware),
            self.graceful,
            self.span,
        )
    }
}
