use std::{
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{extract::Path, routing::get, Extension};
use axum_starter::{
    extension::SetExtension, graceful::SetGraceful, prepare, router::Route, PreparedEffect,
    Provider, ServeAddress, ServerEffect, ServerPrepare,
};
use futures::FutureExt;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    ServerPrepare::with_config(Config {
        id: 11,
        name: "Str".to_string(),
    })
    .append(Student)
    .append(Echo)
    .append(GracefulExit)
    .prepare_start()
    .await
    .expect("")
    .launch()
    .await
    .expect("");
}

#[prepare(Student 'arg)]
async fn arr<'arg>(id: i32, name: &'arg String) -> Result<impl PreparedEffect, Infallible> {
    println!("my name is {name} id is {id}");

    Ok(())
}

#[prepare(Echo)]
fn adding_echo() -> impl PreparedEffect {
    (
        Route::new(
            "/:path",
            get(
                |Path(path): Path<String>, Extension(count): Extension<Arc<AtomicUsize>>| async move {
                    println!("incoming");
                    let now = count.fetch_add(1, Ordering::Relaxed);
                    format!("Welcome {},you are No.{}", path, now + 1)
                },
            ),
        ),
        Route::new("/f/panic", get(|| async { panic!("Not a api") })),
        SetExtension::arc(AtomicUsize::new(0)),
    )
}

#[prepare(GracefulExit)]
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

#[derive(Debug, Provider)]
pub struct Config {
    #[provider(transparent)]
    id: i32,
    #[provider(transparent, ref)]
    name: String,
}

impl ServerEffect for Config {}

impl ServeAddress for Config {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080))
    }
}
