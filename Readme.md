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

use axum::{extract::Path, routing::get};
use axum_starter::{
    graceful::SetGraceful, prepare, router::Route, PreparedEffect, Provider, ServeAddress,
    ServerEffect, ServerPrepare,
};
use futures::FutureExt;
use tokio::sync::oneshot;
use tower_http::trace::TraceLayer;

/// configure for server starter
#[derive(Debug, Provider)]
struct Configure {
    #[provider(ref, transparent)]
    foo: String,
    #[provider(skip)]
    bar: SocketAddr,

    foo_bar: (i32, i32),
}

impl ServeAddress for Configure {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        self.bar
    }
}

impl ServerEffect for Configure {}

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

/// using `#[prepare]`
#[prepare(Logger)]
fn start_logger() -> Result<(), log::SetLoggerError> {
    simple_logger::init()
}

/// if need ref args ,adding a lifetime
#[prepare(ShowFoo 'arg)]
fn show_foo(foo: &'arg String) {
    println!("this is Foo {foo}")
}
#[prepare(EchoRouter)]
fn echo() -> impl PreparedEffect {
    Route::new(
        "/:echo",
        get(|Path(echo): Path<String>| async move { format!("Welcome ! {echo}") }),
    )
}

async fn show(FooBar((x, y)): FooBar) {
    println!("the foo bar is local at ({x}, {y})")
}

/// function style prepare
async fn graceful_shutdown() -> impl PreparedEffect {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            _ => {
                println!("recv ctrl c");
                tx.send(())
            }
        }
    });
    tokio::task::yield_now().await;

    SetGraceful::new(rx.map(|_| ()))
}

#[tokio::main]
async fn main() {
    start().await
}

async fn start() {
    ServerPrepare::with_config(Configure::new())
        .append(Logger)
        .append(ShowFoo)
        .append_fn(show)
        .append_fn(graceful_shutdown)
        .append(EchoRouter)
        .with_global_middleware(TraceLayer::new_for_http())
        .prepare_start()
        .await
        .expect("Prepare for starting server failure")
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
