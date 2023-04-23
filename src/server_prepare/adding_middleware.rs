use tower::layer::util::Stack;

use crate::{prepare_sets::ContainerResult, ServerPrepare};

impl<C: 'static, Log, State, Graceful, R: 'static, L: 'static>
    ServerPrepare<C, ContainerResult<R, L>, Log, State, Graceful>
{
    /// adding middleware without previously [Prepare](crate::Prepare) action
    pub fn layer<M: 'static>(
        self,
        middleware: M,
    ) -> ServerPrepare<C, ContainerResult<R, Stack<M, L>>, Log, State, Graceful> {
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
