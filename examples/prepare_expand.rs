use axum::{
    extract::{FromRef, Path, State},
    routing::get,
};
use axum_starter_macro::Configure;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::FutureExt;
use log::Level;
use tokio::signal::ctrl_c;

use axum_starter::{
    prepare, router::Route, state::AddState, FromStateCollector, PrepareRouteEffect,
    PrepareStateEffect, Provider, ServeAddress, ServerPrepare, StateCollector, TypeNotInState,
};

#[tokio::main]
async fn main() {
    ServerPrepare::with_config(Config {
        id: 11,
        name: "Str".to_string(),
    })
    .init_logger()
    .unwrap_or_else(|e| panic!("init logger panic :{e}"))
    .prepare(Student)
    .prepare_state(EchoState)
    .prepare_route(Echo)
    .graceful_shutdown(ctrl_c().map(|_| ()))
    .convert_state::<MyState>()
    .preparing()
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

#[prepare(box origin Student)]
async fn arr(id: i32, name: &String) {
    println!("my name is {name} id is {id}");
}

#[prepare(EchoState)]
fn echo_count() -> impl PrepareStateEffect {
    AddState::new(Arc::new(AtomicUsize::new(0)))
}

#[prepare(sync Echo)]
fn adding_echo<S>() -> impl PrepareRouteEffect<S>
where
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

#[derive(Debug, Provider, Configure)]
#[conf(
    logger(
        error = "log::SetLoggerError",
        func = "||simple_logger::init_with_level(Level::Info)",
        associate
    ),
    server
)]
pub struct Config {
    #[provider(transparent)]
    id: i32,
    #[provider(transparent, r#ref)]
    name: String,
}

impl ServeAddress for Config {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8000)
    }
}
