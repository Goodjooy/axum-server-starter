use std::future::Future;

use axum::extract::FromRef;
use futures::future::BoxFuture;

use crate::server_prepare::state_ready::StateReady;
use crate::ServerPrepare;

pub trait PostPrepare<S, Args>
where
    S: Send + Sync + 'static,
{
    type PostFut: Future<Output = ()> + Send + 'static;

    fn exec(self, state: &S) -> Self::PostFut;
}

pub type PostPrepareFn<S> = Box<dyn FnOnce(S) -> BoxFuture<'static, ()>+Send>;
fn post_prepare_to_dyn<S, Args, T>(prepare: T) -> PostPrepareFn<S>
where
    S: Send + Sync + 'static,
    Args: Send + 'static,
    T: PostPrepare<S, Args> + Send + 'static,
{
    Box::new(move |s: S| {
        Box::pin(async move { <T as PostPrepare<S, Args>>::exec(prepare, &s).await })
    })
}

macro_rules! post_prepare_gen {
    ($($args:ident),*) => {
        impl<T, S, $($args,)* Fut> PostPrepare<S, ($($args,)* )> for T
            where
                T: FnOnce($($args),*) -> Fut,
                Fut: Future<Output=()> + Send + 'static,
                S:Send+Sync+'static,
                $(
                $args: FromRef<S>
                ),*
        {
            type PostFut = Fut;
                
            #[allow(unused_variables)]
            fn exec(self, state: &S) -> Self::PostFut {
                (self)(
                 $(<$args as FromRef<S>>::from_ref(state)),*
                )
            }
        }
    };
}
post_prepare_gen!();
post_prepare_gen!(A0);
post_prepare_gen!(A0, A1);
post_prepare_gen!(A0, A1, A2);
post_prepare_gen!(A0, A1, A2, A3);
post_prepare_gen!(A0, A1, A2, A3, A4);
post_prepare_gen!(A0, A1, A2, A3, A4, A5);
post_prepare_gen!(A0, A1, A2, A3, A4, A5, A6);
post_prepare_gen!(A0, A1, A2, A3, A4, A5, A6, A7);
post_prepare_gen!(A0, A1, A2, A3, A4, A5, A6, A7, A8);
post_prepare_gen!(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9);
post_prepare_gen!(A0, A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);

impl<C, Effect, Log, State, Graceful, Decorator>
    ServerPrepare<C, Effect, Log, StateReady<State>, Graceful, Decorator>
{
    
    /// execute a task after all prepare task done before service start
    /// 
    /// the task can be a [FnOnce] which has the following features
    /// 1. the arg list all impl Arg: From<State>
    /// 2. is an Async Function
    /// 3. the function return `()`
    /// 
    /// # Note
    /// those tasks will run in a spawn tokio task, do not assume the service has been started
    pub fn post_prepare<Args, T>(mut self, post_prepare: T) -> Self
    where
        T: PostPrepare<State, Args> + Send + 'static,
        State: Send + 'static + Sync,
        Args: Send + 'static,
    {
        self.state.push(post_prepare_to_dyn(post_prepare));
        self
    }
}
