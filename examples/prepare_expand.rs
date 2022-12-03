use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{
    extract::{FromRef, Path, State},
    routing::get,
};

use axum_starter::{
    prepare, router::Route, state::AddState, ConfigureServerEffect, FromStateCollector,
    PrepareRouteEffect, PrepareStateEffect, Provider, ServeAddress, ServerPrepare, StateCollector,
    TypeNotInState,
};
use futures::FutureExt;
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
    ServerPrepare::with_config(Config {
        id: 11,
        name: "Str".to_string(),
    })
    .prepare_route(Student)
    .prepare_state(EchoState)
    .prepare_route(Echo)
    .graceful_shutdown(ctrl_c().map(|_| ()))
    .convert_state::<MyState>()
    .prepare_start()
    .await
    .expect("")
    .launch()
    .await
    .expect("");
}

#[derive(Debug, Clone)]
struct MyState {
    count: Arc<AtomicUsize>,
}

impl FromRef<MyState> for Arc<AtomicUsize> {
    fn from_ref(input: &MyState) -> Self {
        Arc::clone(&input.count)
    }
}

impl FromStateCollector for MyState {
    fn fetch_mut(collector: &mut StateCollector) -> Result<Self, TypeNotInState> {
        Ok(Self {
            count: collector.take()?,
        })
    }
}

#[prepare(box Student 'arg)]
async fn arr<'arg>(id: i32, name: &'arg String) {
    println!("my name is {name} id is {id}");
}

#[prepare(EchoState)]
fn echo_count() -> impl PrepareStateEffect {
    AddState::new(Arc::new(AtomicUsize::new(0)))
}

#[prepare(Echo)]
fn adding_echo<B, S>() -> impl PrepareRouteEffect<S, B>
where
    B: http_body::Body + Send + 'static,
    S: Clone + Send + Sync + 'static,
    Arc<AtomicUsize>: FromRef<S>,
{
    (
        Route::new(
            "/:path",
            get(
                |Path(path): Path<String>, State(count): State<Arc<AtomicUsize>>| async move {
                    println!("incoming");
                    let now = count.fetch_add(1, Ordering::Relaxed);
                    format!("Welcome {},you are No.{}", path, now + 1)
                },
            ),
        ),
        Route::new("/f/panic", get(|| async { panic!("Not a api") })),
    )
}

#[derive(Debug, Provider)]
pub struct Config {
    #[provider(transparent)]
    id: i32,
    #[provider(transparent, ref)]
    name: String,
}

impl ConfigureServerEffect for Config {}

impl ServeAddress for Config {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080))
    }
}
