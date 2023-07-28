use tower::layer::util::Stack;

use crate::{prepare_sets::ContainerResult, ServerPrepare};
use crate::server_prepare::PrepareDecorator;

type MiddlewareLayerRet<C, R, M, L, Log, State, Graceful, Decorator> = ServerPrepare<C, ContainerResult<R, Stack<M, L>>, Log, State, Graceful, Decorator>;

impl<C: 'static, Log, State, Graceful, R: 'static, L: 'static, Decorator>
ServerPrepare<C, ContainerResult<R, L>, Log, State, Graceful, Decorator>
where Decorator:PrepareDecorator
{
    /// adding middleware without previously [Prepare](crate::Prepare) action
    pub fn layer<M: 'static>(
        self,
        middleware: M,
    ) -> MiddlewareLayerRet<C, R, M, L, Log, State, Graceful, Decorator> {
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
