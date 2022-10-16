use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use axum::{extract::Path, routing::get};
use axum_starter::{
    graceful::SetGraceful, prepare, router::Route, ConfigureServerEffect, PreparedEffect, Provider,
    ServeAddress, ServerPrepare,
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
        .expect("Prepare for starting server failure ")
        .launch()
        .await
        .expect("Server Error")
}
