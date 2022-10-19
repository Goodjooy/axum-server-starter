use std::{convert::Infallible, error, pin::Pin};

use axum::Router;
use futures::Future;
use hyper::server::conn::AddrIncoming;

use crate::{ExtensionEffect, GracefulEffect, PreparedEffect, RouteEffect, ServerEffect};

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

pub struct EffectsCollect<Route = (), Graceful = (), Extension = (), Server = ()>
where
    Route: RouteEffect,
    Graceful: GracefulEffect,
    Extension: ExtensionEffect,
    Server: ServerEffect,
{
    route: Route,
    graceful: Graceful,
    extension: Extension,
    server: Server,
}

impl<Route, Graceful, Extension, Server> PreparedEffect
    for EffectsCollect<Route, Graceful, Extension, Server>
where
    Route: RouteEffect,
    Graceful: GracefulEffect,
    Extension: ExtensionEffect,
    Server: ServerEffect,
{
    type Extension = Extension;

    type Graceful = Graceful;

    type Route = Route;

    type Server = Server;

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        (extension, route, graceful, server)
    }
}

impl<Route, Graceful, Extension, Server> EffectsCollect<Route, Graceful, Extension, Server>
where
    Route: RouteEffect,
    Graceful: GracefulEffect,
    Extension: ExtensionEffect,
    Server: ServerEffect,
{
    pub fn with_route<R: RouteEffect>(
        self,
        new_route: R,
    ) -> EffectsCollect<(Route, R), Graceful, Extension, Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollect {
            route: (route, new_route),
            graceful,
            extension,
            server,
        }
    }

    pub fn with_extension<E: ExtensionEffect>(
        self,
        new_extension: E,
    ) -> EffectsCollect<Route, Graceful, (Extension, E), Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollect {
            route,
            graceful,
            extension: (extension, new_extension),
            server,
        }
    }

    pub fn with_server<S: ServerEffect>(
        self,
        new_server: S,
    ) -> EffectsCollect<Route, Graceful, Extension, (Server, S)> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollect {
            route,
            graceful,
            extension,
            server: (server, new_server),
        }
    }
    pub fn with_graceful<G: GracefulEffect>(
        self,
        new_graceful: G,
    ) -> EffectsCollect<Route, (Graceful, G), Extension, Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollect {
            route,
            graceful: (graceful, new_graceful),
            extension,
            server,
        }
    }

    pub fn with_effect<E: PreparedEffect>(
        self,
        effect: E,
    ) -> EffectsCollect<
        (Route, E::Route),
        (Graceful, E::Graceful),
        (Extension, E::Extension),
        (Server, E::Server),
    > {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        let effect = effect.split_effect();

        EffectsCollect {
            route: (route, effect.1),
            graceful: (graceful, effect.2),
            extension: (extension, effect.0),
            server: (server, effect.3),
        }
    }
}

impl EffectsCollect {
    pub fn new() -> Self {
        Self {
            route: (),
            graceful: (),
            extension: (),
            server: (),
        }
    }
}

pub fn extension_only<S: PreparedEffect<Graceful = (), Route = (), Server = ()>>(
    extension: S::Extension,
) -> (S::Extension, S::Route, S::Graceful, S::Server) {
    (extension, (), (), ())
}
pub fn graceful_only<S: PreparedEffect<Extension = (), Route = (), Server = ()>>(
    graceful: S::Graceful,
) -> (S::Extension, S::Route, S::Graceful, S::Server) {
    ((), (), graceful, ())
}
pub fn route_only<S: PreparedEffect<Graceful = (), Server = (), Extension = ()>>(
    route: S::Route,
) -> (S::Extension, S::Route, S::Graceful, S::Server) {
    ((), route, (), ())
}
pub fn serve_only<S: PreparedEffect<Graceful = (), Route = (), Extension = ()>>(
    server: S::Server,
) -> (S::Extension, S::Route, S::Graceful, S::Server) {
    ((), (), (), server)
}

macro_rules! group_prepared_effect {
    ($($args:ident),*$(,)?) => {
        impl<$($args),*> PreparedEffect for ($($args,)*)
        where
            $(
                $args: PreparedEffect,
            )*
        {
            type Extension = ($(<$args as PreparedEffect>::Extension,)*);

            type Graceful = ($(<$args as PreparedEffect>::Graceful,)*);

            type Route = ($(<$args as PreparedEffect>::Route,)*);

            type Server = ($(<$args as PreparedEffect>::Server,)*);

            #[allow(non_snake_case)]
            fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
                let ($($args,)*) = self;
                $(
                    let $args = PreparedEffect::split_effect($args);
                )*
                (
                    ($($args.0,)*),
                    ($($args.1,)*),
                    ($($args.2,)*),
                    ($($args.3,)*),
                )
            }
        }

        impl<$($args),*> ExtensionEffect for ($($args,)*)
        where
        $(
            $args: ExtensionEffect,
        )*
        {
            #[allow(non_snake_case)]
            fn add_extension(self, extension: crate::ExtensionManage) -> crate::ExtensionManage {
                let ($($args,)*) = self;
                $(
                    let extension = ExtensionEffect::add_extension($args, extension);
                )*
                extension
            }
        }
        impl<$($args),*> GracefulEffect for ($($args,)*)
        where
        $(
            $args: GracefulEffect,
        )*
        {
            #[allow(non_snake_case)]
            fn set_graceful(self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
                let ($($args,)*) = self;
                let ret = None;
                $(
                    let ret = match (ret, GracefulEffect::set_graceful($args)) {
                        (None, None) => None,
                        (None, ret @ Some(_)) | (ret @ Some(_), _) => ret,
                    };
                )*
                ret
            }
        }

        impl<$($args),*> RouteEffect for ($($args,)*)
        where
        $(
            $args: RouteEffect,
        )*
        {
            #[allow(non_snake_case)]
            fn add_router(self, router: Router) -> Router {
                let ($($args,)*) = self;
                $(
                    let router = RouteEffect::add_router($args, router);
                )*
                router
            }
        }

        impl<$($args),*> ServerEffect for ($($args,)*)
        where
        $(
            $args: ServerEffect,
        )*
        {
            #[allow(non_snake_case)]
            fn config_serve(
                self,
                server: hyper::server::Builder<AddrIncoming>,
            ) -> hyper::server::Builder<AddrIncoming> {
                let ($($args,)*) = self;
                $(
                    let server = ServerEffect::config_serve($args,server);
                )*
                server
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
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
group_prepared_effect!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y);
group_prepared_effect!(
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z
);
