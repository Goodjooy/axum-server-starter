use std::{error, marker::PhantomData, sync::Arc};

use futures::{Future, TryFutureExt};
use tap::Pipe;

use crate::{FromConfig, Prepare, PreparedEffect};

pub trait PrepareHandler<Args, C> {
    type Effect: PreparedEffect + 'static;
    type Error: std::error::Error + 'static;
    type Future: Future<Output = Result<Self::Effect, Self::Error>> + 'static;
    fn prepare(self, config: Arc<C>) -> Self::Future;
}

pub fn fn_prepare<C, Args, F>(func: F) -> FnPrepare<C, Args, F>
where
    F: PrepareHandler<Args, C>,
{
    FnPrepare(func, PhantomData)
}

pub struct FnPrepare<C, Args, F>(F, PhantomData<(Args, C)>)
where
    F: PrepareHandler<Args, C>;

impl<C, Args, F> Prepare<C> for FnPrepare<C, Args, F>
where
    F: PrepareHandler<Args, C>,
{
    fn prepare(self, config: Arc<C>) -> crate::BoxPreparedEffect {
        self.0
            .prepare(config)
            .map_ok(|effect| Box::new(effect) as Box<dyn PreparedEffect>)
            .map_err(|err| Box::new(err) as Box<dyn error::Error>)
            .pipe(Box::pin)
    }
}

macro_rules! fn_prepare_handles {
    ($($args:ident),* $(,)?) => {
        impl<C, F, Fut, P, E, $($args),*> PrepareHandler<($($args,)*), C> for F
        where
            F: FnOnce($($args),*) -> Fut + 'static,
            Fut: Future<Output = Result<P, E>> + 'static,
            P: PreparedEffect + 'static,
            E: std::error::Error + 'static,
            $(
                $args: for<'r>FromConfig<'r, C>
            ),*
        {
            type Effect = P;

            type Error = E;

            type Future = Fut;

            #[allow(unused_variables)]
            fn prepare(self, config: Arc<C>) -> Self::Future {
                self(
                    $(
                        <$args as FromConfig<C>>::from_config(&config)
                    ),*
                )
            }
        }

    };
}

fn_prepare_handles!();
fn_prepare_handles!(T1);
fn_prepare_handles!(T1, T2);
fn_prepare_handles!(T1, T2, T3);
fn_prepare_handles!(T1, T2, T3, T4);
fn_prepare_handles!(T1, T2, T3, T4, T5);
fn_prepare_handles!(T1, T2, T3, T4, T5, T6,);
fn_prepare_handles!(T1, T2, T3, T4, T5, T6, T7);
fn_prepare_handles!(T1, T2, T3, T4, T5, T6, T7, T8);
fn_prepare_handles!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
fn_prepare_handles!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
