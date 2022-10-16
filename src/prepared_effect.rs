use std::{convert::Infallible, error};

use axum::Router;
use hyper::server::{conn::AddrIncoming, Builder};

use crate::PreparedEffect;

/// fallible prepare effect
pub trait IntoFallibleEffect {
    type Effect: PreparedEffect;
    type Error: std::error::Error;

    fn into_effect(self) -> Result<Self::Effect, Self::Error>;
}

pub fn into_effect<T: IntoFallibleEffect + 'static>(this: T) -> Result<T::Effect, T::Error> {
    IntoFallibleEffect::into_effect(this)
}

impl<T: PreparedEffect, E: error::Error> IntoFallibleEffect for Result<T, E> {
    type Effect = T;

    type Error = E;

    fn into_effect(self) -> Result<Self::Effect, Self::Error> {
        self
    }
}
impl<T: PreparedEffect> IntoFallibleEffect for T {
    type Effect = T;

    type Error = Infallible;

    fn into_effect(self) -> Result<Self::Effect, Self::Error> {
        Ok(self)
    }
}

macro_rules! group_prepared_effect {
    ($($args:ident),*$(,)?) => {
        impl<$($args),*> PreparedEffect for ($($args,)*)
        where
            $(
                $args: PreparedEffect,
            )*
        {
            #[allow(non_snake_case)]
            fn add_extension(&mut self, extension: crate::ExtensionManage) -> crate::ExtensionManage {
                let ($($args,)*) = self;
                $(
                    let extension = PreparedEffect::add_extension($args, extension);
                )*
                extension
            }
            #[allow(non_snake_case)]
            fn set_graceful(&mut self) -> Option<std::pin::Pin<Box<dyn futures::Future<Output = ()>>>> {
                let ($($args,)*) = self;
                let ret = None;
                $(
                    let ret = match (ret, PreparedEffect::set_graceful($args)) {
                        (None, None) => None,
                        (None, ret @ Some(_)) | (ret @ Some(_), _) => ret,
                    };
                )*
                ret
            }
            #[allow(non_snake_case)]
            fn config_serve(&self, server: Builder<AddrIncoming>) -> Builder<AddrIncoming> {
                let ($($args,)*) = self;
                $(
                    let server = PreparedEffect::config_serve($args,server);
                )*
                server
            }
            #[allow(non_snake_case)]
            fn add_router(&mut self, router: Router) -> Router {
                let ($($args,)*) = self;
                $(
                    let router = PreparedEffect::add_router($args,router);
                )*
                router
            }
        }

    };
}

group_prepared_effect!();
group_prepared_effect!(A);
group_prepared_effect!(A, B);
group_prepared_effect!(A, B, C);
group_prepared_effect!(A, B, C, D);
group_prepared_effect!(A, B, C, D, E);
group_prepared_effect!(A, B, C, D, E, F);
group_prepared_effect!(A, B, C, D, E, F, G);
group_prepared_effect!(A, B, C, D, E, F, G, H);
group_prepared_effect!(A, B, C, D, E, F, G, H, I);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
