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

pub struct EffectsCollector<Route = (), Graceful = (), Extension = (), Server = ()> {
    route: Route,
    graceful: Graceful,
    extension: Extension,
    server: Server,
}

impl<Route, Graceful, Extension, Server: ServerEffect> ServerEffect
    for EffectsCollector<Route, Graceful, Extension, Server>
{
    fn config_serve(
        self,
        server: hyper::server::Builder<AddrIncoming>,
    ) -> hyper::server::Builder<AddrIncoming> {
        self.server.config_serve(server)
    }
}

impl<Route, Graceful, Extension: ExtensionEffect, Server> ExtensionEffect
    for EffectsCollector<Route, Graceful, Extension, Server>
{
    fn add_extension(self, extension: crate::ExtensionManage) -> crate::ExtensionManage {
        self.extension.add_extension(extension)
    }
}

impl<Route, Graceful: GracefulEffect, Extension, Server> GracefulEffect
    for EffectsCollector<Route, Graceful, Extension, Server>
{
    fn set_graceful(self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        self.graceful.set_graceful()
    }
}

impl<Route, Graceful, Extension, Server> RouteEffect
    for EffectsCollector<Route, Graceful, Extension, Server>
where
    Route: RouteEffect,
{
    fn add_router(self, router: Router) -> Router {
        self.route.add_router(router)
    }
}

impl<Route, Graceful, Extension, Server> PreparedEffect
    for EffectsCollector<Route, Graceful, Extension, Server>
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

impl<Route, Graceful, Extension, Server> EffectsCollector<Route, Graceful, Extension, Server>
where
    Route: RouteEffect,
    Graceful: GracefulEffect,
    Extension: ExtensionEffect,
    Server: ServerEffect,
{
    pub fn with_route<R: RouteEffect>(
        self,
        new_route: R,
    ) -> EffectsCollector<(Route, R), Graceful, Extension, Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollector {
            route: (route, new_route),
            graceful,
            extension,
            server,
        }
    }

    pub fn with_extension<E: ExtensionEffect>(
        self,
        new_extension: E,
    ) -> EffectsCollector<Route, Graceful, (Extension, E), Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollector {
            route,
            graceful,
            extension: (extension, new_extension),
            server,
        }
    }

    pub fn with_server<S: ServerEffect>(
        self,
        new_server: S,
    ) -> EffectsCollector<Route, Graceful, Extension, (Server, S)> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollector {
            route,
            graceful,
            extension,
            server: (server, new_server),
        }
    }
    pub fn with_graceful<G: GracefulEffect>(
        self,
        new_graceful: G,
    ) -> EffectsCollector<Route, (Graceful, G), Extension, Server> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        EffectsCollector {
            route,
            graceful: (graceful, new_graceful),
            extension,
            server,
        }
    }

    pub fn with_effect<E: PreparedEffect>(
        self,
        effect: E,
    ) -> CombineEffects<Route, Graceful, Extension, Server, E> {
        let Self {
            route,
            graceful,
            extension,
            server,
        } = self;
        let effect = effect.split_effect();

        EffectsCollector {
            route: (route, effect.1),
            graceful: (graceful, effect.2),
            extension: (extension, effect.0),
            server: (server, effect.3),
        }
    }
}

type CombineEffects<Route, Graceful, Extension, Server, E> = EffectsCollector<
    (Route, <E as PreparedEffect>::Route),
    (Graceful, <E as PreparedEffect>::Graceful),
    (Extension, <E as PreparedEffect>::Extension),
    (Server, <E as PreparedEffect>::Server),
>;

impl Default for EffectsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl EffectsCollector {
    pub fn new() -> Self {
        Self {
            route: (),
            graceful: (),
            extension: (),
            server: (),
        }
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
