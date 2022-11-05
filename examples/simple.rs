use std::{
    fmt::Debug,
    iter::Cloned,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
};

use axum::{
    extract::{OriginalUri, Path},
    handler::Handler,
    routing::get,
    Router,
};
use axum_starter::{
    graceful::SetGraceful,
    prepare,
    router::{Fallback, Nest, Route},
    EffectsCollector, PreparedEffect, Provider, ServerPrepare,
};
use axum_starter_macro::Configure;
use futures::FutureExt;

use std::slice::Iter;
use tower_http::trace::TraceLayer;
/// configure for server starter
#[derive(Debug, Provider, Configure)]
#[conf(
    address(provide),
    logger(error = "log::SetLoggerError", func = "simple_logger::init", associate),
    server
)]
struct Configure {
    #[provider(ref, transparent)]
    #[provider(map_to(ty = "&'s str", by = "String::as_str", lifetime = "'s"))]
    #[provider(map_to(ty = "String", by = "Clone::clone"))]
    foo: String,
    #[provider(transparent)]
    bar: SocketAddr,

    foo_bar: (i32, i32),
    #[provider(
        transparent,
        map_to(ty = "Cloned<Iter<'a, i32>>", by = "vec_iter", lifetime = "'a")
    )]
    iter: Vec<i32>,
}

fn vec_iter<T: std::clone::Clone>(vec: &Vec<T>) -> Cloned<Iter<'_, T>> {
    vec.iter().cloned()
}

impl Configure {
    pub fn new() -> Self {
        Self {
            foo: "Foo".into(),
            bar: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080)),
            foo_bar: (1, 2),
            iter: vec![1, 2, 3, 4, 5, 6, 7, 8],
        }
    }
}
// prepares

#[prepare(LifetimeBound 'arg)]
fn lifetime_bound<'arg, I>(iter: I)
where
    I: IntoIterator<Item = i32>,
{
    for i in iter {
        log::info!("Has config iter {:?}", i)
    }
}

/// if need ref args ,adding a lifetime
#[prepare(box ShowFoo 'arg)]
fn show_foo<S>(f: &'arg S)
where
    S: AsRef<str> + ?Sized,
{
    println!("this is Foo {}", f.as_ref())
}

#[prepare(ShowValue)]
fn the_value<const V: i32>() {
    println!("The value is {}", V)
}

/// using `#[prepare]`
#[prepare(EchoRouter)]
fn echo() -> impl PreparedEffect {
    Route::new(
        "/:echo",
        get(|Path(echo): Path<String>| async move { format!("Welcome ! {echo}") }),
    )
}
#[prepare(box C)]
fn routers() -> impl PreparedEffect {
    EffectsCollector::new()
        .with_route(Nest::new(
            "/aac/b",
            Router::new().route(
                "/a",
                get(|OriginalUri(uri): OriginalUri| async move { format!("welcome {uri}") }),
            ),
        ))
        .with_route(Fallback::new(Handler::into_service(|| async { "oops" })))
        .with_server(axum_starter::service::ConfigServer::new(|s| {
            s.http1_only(true)
        }))
}

#[prepare(box Show)]
async fn show(FooBar((x, y)): FooBar) {
    println!("the foo bar is local at ({x}, {y})")
}

/// function style prepare
async fn graceful_shutdown() -> impl PreparedEffect {
    SetGraceful::new(
        tokio::signal::ctrl_c()
            .map(|_| println!("recv Exit msg"))
            .map(|_| ()),
    )
}

#[tokio::main]
async fn main() {
    start().await
}

async fn start() {
    ServerPrepare::with_config(Configure::new())
        .init_logger()
        .expect("Init Logger Failure")
        .append(ShowValue::<_, 11>)
        // .append(LifetimeBound::<_, Cloned<Iter<i32>>>)
        .append_concurrent(|set| set.join(ShowFoo::<_, String>).join(C).join(Show))
        .append_fn(graceful_shutdown)
        .append(EchoRouter)
        .with_global_middleware(TraceLayer::new_for_http())
        .prepare_start()
        .await
        .expect("Prepare for starting server failure ")
        .launch()
        .await
        .expect("Server Error")
}
