use std::{
    any::Any,
    future::Future,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{
    body::BoxBody,
    extract::Path,
    response::IntoResponse,
    routing::{get, MethodRouter},
    Extension,
};
use axum_starter::{
    extension::SetExtension, graceful::SetGraceful, router::Route, ConfigureServerEffect,
    PreparedEffect, Provider, ServeAddress, ServerPrepare,
};
use hyper::Response;
use tokio::sync::oneshot;
use tower_http::catch_panic::CatchPanicLayer;
#[tokio::main]
async fn main() {
    ServerPrepare::with_config(Config::new())
        .append_fn(ctrl_c_stop)
        .append_fn(echo_handler)
        .append_fn(show_address)
        .append_fn(print_init)
        .append_fn(show_my_info)
        .with_global_middleware(CatchPanicLayer::custom(serve_panic))
        .prepare_start()
        .await
        .expect("准备启动服务异常")
        .launch()
        .await
        .expect("Server Error");
}

fn serve_panic(_: Box<dyn Any + Send + 'static>) -> Response<BoxBody> {
    "Panic".into_response()
}

#[derive(Debug, Provider)]
struct Config {
    pwd: String,
    name: String,
    id: u64,
}

impl Config {
    fn new() -> Self {
        Self {
            pwd: "Foo".into(),
            name: "Bar".into(),
            id: 114514,
        }
    }
}

async fn show_my_info(Pwd(pwd): Pwd, Name(name): Name, Id(id): Id) {
    println!("my name is {name} pwd is {pwd} id is {id}")
}

impl<'r> Provider<'r, SocketAddr> for Config {
    fn provide(&'r self) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080))
    }
}

impl ServeAddress for Config {
    type Address = SocketAddr;

    fn get_address(&self) -> Self::Address {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080))
    }
}

impl ConfigureServerEffect for Config {}

fn show_address(addr: SocketAddr) -> impl Future<Output = ()> {
    async move {
        println!("server serve at http://{:?}", addr);
    }
}

async fn echo_handler() -> EchoEffect {
    EchoEffect
}

struct EchoEffect;
impl PreparedEffect for EchoEffect {
    type Extension = SetExtension<Arc<AtomicUsize>>;

    type Graceful = ();

    type Route = (Route<MethodRouter>, Route<MethodRouter>);

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        (
            SetExtension::arc(AtomicUsize::new(0)),
            (
                Route::new(
                    "/:path",
                    get(
                        |Path(path): Path<String>,
                         Extension(count): Extension<Arc<AtomicUsize>>| async move {
                            println!("incoming");
                            let now = count.fetch_add(1, Ordering::Relaxed);
                            format!("Welcome {},you are No.{}", path, now + 1)
                        },
                    ),
                ),
                Route::new("/f/panic", get(|| async { panic!("Not a api") })),
            ),
            (),
            (),
        )
    }
}

async fn print_init() {
    println!("Initial");
}

async fn ctrl_c_stop() -> CtrlCEffect<impl Future<Output = ()>> {
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

    let fut = async move {
        rx.await.ok();
        println!("recv ctrl c");
    };
    CtrlCEffect { fut }
}

struct CtrlCEffect<F: Future<Output = ()>> {
    fut: F,
}

impl<F: Future<Output = ()> + 'static> PreparedEffect for CtrlCEffect<F> {
    type Extension = ();

    type Graceful = SetGraceful;

    type Route = ();

    type Server = ();

    fn split_effect(self) -> (Self::Extension, Self::Route, Self::Graceful, Self::Server) {
        ((), (), SetGraceful::new(self.fut), ())
    }
}
