use std::{error, marker::PhantomData, sync::Arc};

use futures::{Future, FutureExt, TryFutureExt};
use tap::Pipe;

use crate::{into_effect::IntoFallibleEffect, FromConfig, Prepare, PreparedEffect};

pub trait PrepareHandler<Args, C> {
    type IntoEffect: IntoFallibleEffect + 'static;
    type Future: Future<Output = Self::IntoEffect> + 'static;
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
            .map(|fut| fut.into_effect())
            .map_ok(|effect| Box::new(effect) as Box<dyn PreparedEffect>)
            .map_err(|err| Box::new(err) as Box<dyn error::Error>)
            .pipe(Box::pin)
    }
}

macro_rules! fn_prepare_handles {
    ($($args:ident),* $(,)?) => {
        impl<Config, Func, Fut, FallibleEffect, $($args),*> PrepareHandler<($($args,)*), Config> for Func
        where
            Func: FnOnce($($args),*) -> Fut + 'static,
            Fut: Future<Output = FallibleEffect> + 'static,
            FallibleEffect: IntoFallibleEffect + 'static,
            $(
                $args: for<'r>FromConfig<'r, Config>
            ),*
        {
            type IntoEffect = FallibleEffect;

            type Future = Fut;

            #[allow(unused_variables)]
            fn prepare(self, config: Arc<Config>) -> Self::Future {
                self(
                    $(
                        <$args as FromConfig<Config>>::from_config(&config)
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
