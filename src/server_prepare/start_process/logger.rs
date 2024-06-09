use crate::{LoggerInitialization, ServerPrepare};

pub struct NoLog;

pub struct LogInit;

type LogResult<C, FutEffect, LogInit, State, Graceful, Decorator> = Result<
    ServerPrepare<C, FutEffect, LogInit, State, Graceful, Decorator>,
    <C as LoggerInitialization>::Error,
>;

impl<C, FutEffect, State, Graceful, Decorator>
    ServerPrepare<C, FutEffect, NoLog, State, Graceful, Decorator>
where
    C: LoggerInitialization,
{
    /// init the (logger) of this [ServerPrepare] ,require C impl [LoggerInitialization]
    pub fn init_logger(self) -> LogResult<C, FutEffect, LogInit, State, Graceful, Decorator> {
        self.span.in_scope(|| {
            let t = self.prepares.get_ref_configure().init_logger();
            info!(logger = "Init");
            t
        })?;

        Ok(ServerPrepare::new(
            self.prepares,
            self.graceful,
            self.state,
            self.span,
        ))
    }
}
