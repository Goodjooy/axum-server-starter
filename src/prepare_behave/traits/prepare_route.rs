use axum::Router;


/// prepare side effect of [Router](Router)
pub trait PrepareRouteEffect<S>: 'static + Sized {
    fn set_route(self, route: Router<S>) -> Router<S>
    where
        S: Clone + Send + Sync + 'static;
}

macro_rules! route_effect {
    ($($id:ident),* $(,)?) => {
        impl<$($id,)* S> PrepareRouteEffect<S> for ($($id,)*)
        where
            $(
                $id: PrepareRouteEffect<S>,
            )*
        {
            #[allow(non_snake_case)]
            fn set_route(self, route: Router<S>) -> Router<S>
            where
                S: Clone + Send + Sync + 'static
            {
                let ($($id,)*) = self;

                $(
                    let route = $id.set_route(route);
                )*

                route
            }
        }
    };
}
route_effect!();
route_effect!(T1);
route_effect!(T1, T2);
route_effect!(T1, T2, T3);
route_effect!(T1, T2, T3, T4);
route_effect!(T1, T2, T3, T4, T5);
route_effect!(T1, T2, T3, T4, T5, T6);
route_effect!(T1, T2, T3, T4, T5, T6, T7);
route_effect!(T1, T2, T3, T4, T5, T6, T7, T8);
