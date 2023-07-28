use crate::server_prepare::PrepareDecorator;
use crate::ServerPrepare;

impl<C, Effect, Log , State , Graceful ,Decorator> ServerPrepare<C, Effect, Log, State, Graceful, Decorator> {
    /// Add Decorator apply on every prepare [`futures::Future`]
    ///
    /// this will overwrite old [`PrepareDecorator`], combine multiply [`PrepareDecorator`] is **Not**
    ///support. Manual writing combining code instead
    pub fn set_decorator<D>(self,_:D)->ServerPrepare<C, Effect, Log, State, Graceful, D>
    where D:PrepareDecorator
    {
        ServerPrepare::new(self.prepares.change_decorator(),self.graceful,self.span)
    }
}