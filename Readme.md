# Axum Starter

## Why axum-starter

With the growing of the server functions, the code which prepare multiply infrastructures for the server in the main become more and more complex.  
For example, I need connect to `Mysql` and `Redis`, start `MessageQuery` , start GracefulShutdown and so on.  
In other to simplify the start up code with my server project, there comes the `axum-starter`

## Quick Start

The following example using `axum-starter` starting a web server which
server on `http://127.0.0.1:8080`

It can do

1. show info before launch
2. using `simple_logger` and adding TraceLayer as logger middleware
3. request `http://127.0.0.1:8080/{name}` will respond greet with your name
4. using `ctrl + c` can graceful stop the server

```rust

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

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
    ConfigureServerEffect, EffectsCollector, LoggerInitialization, PreparedEffect, Provider,
    ServeAddress, ServerPrepare,
};
use futures::FutureExt;

use tower_http::trace::TraceLayer;

/// configure for server starter
#[derive(Debug, Provider)]
struct Configure {
    #[provider(ref, transparent)]
    #[provider(map_to(ty = "&'s str", by = "String::as_str", lifetime = "'s"))]
    #[provider(map_to(ty = "String", by = "Clone::clone"))]
    foo: String,
    #[provider(skip)]
    bar: SocketAddr,

    foo_bar: (i32, i32),
}

impl LoggerInitialization for Configure {
    type Error = log::SetLoggerError;

    fn init_logger(&self) -> Result<(), Self::Error> {
        simple_logger::init()
    }
}

impl ServeAddress for Configure {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        self.bar
    }
}

impl ConfigureServerEffect for Configure {}

impl Configure {
    pub fn new() -> Self {
        Self {
            foo: "Foo".into(),
            bar: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080)),
            foo_bar: (1, 2),
        }
    }
}
// prepares

/// if need ref args ,adding a lifetime
#[prepare(ShowFoo 'arg)]
fn show_foo(f: &'arg String) {
    println!("this is Foo {f}")
}
/// using `#[prepare]`
#[prepare(EchoRouter)]
fn echo() -> impl PreparedEffect {
    Route::new(
        "/:echo",
        get(|Path(echo): Path<String>| async move { format!("Welcome ! {echo}") }),
    )
}
#[prepare(C)]
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
        .append(ShowFoo)
        .append(C)
        .append_fn(show)
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

```

### `Prepare` trait

the trait define how to apply the prepare task,
after prepare down, it return a `PreparedEffect`

### `PreparedEffect` trait

the trait will apply multiply effect on the server. include the following

- Router
- Extension
- GracefulShutdown
- setting the internal `hyper::Server`
