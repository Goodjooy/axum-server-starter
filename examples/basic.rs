use std::{
    any::Any,
    future::Future,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    pin::Pin,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{body::BoxBody, extract::Path, response::IntoResponse, routing::get, Extension};
use axum_starter::{
    ExtensionManage, PreparedEffect, Provider, ServeAddress, ServerEffect, ServerPrepare,
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

impl ServerEffect for Config {}

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
    fn add_extension(&mut self, extension: ExtensionManage) -> ExtensionManage {
        let state = Arc::new(AtomicUsize::new(0));

        extension.add_extension(state)
    }

    fn add_router(&mut self, router: axum::Router) -> axum::Router {
        router.route(
            "/:path",
            get(
                |Path(path): Path<String>, Extension(count): Extension<Arc<AtomicUsize>>| async move{
                    println!("incoming");
                    let now = count.fetch_add(1, Ordering::Relaxed);
                    format!("Welcome {},you are No.{}", path, now+1)
                },
            ),
        ).route("/f/panic",get(|| async{panic!("Not a api")}))
    }
}

async fn print_init() {
    println!("Initial");
}

async fn ctrl_c_stop() -> CtrlCEffect {
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

    let fut = Box::pin(async move {
        rx.await.ok();
        println!("recv ctrl c");
    });
    CtrlCEffect { fut: Some(fut) }
}

struct CtrlCEffect {
    fut: Option<Pin<Box<dyn Future<Output = ()>>>>,
}

impl PreparedEffect for CtrlCEffect {
    fn set_graceful(&mut self) -> Option<Pin<Box<dyn Future<Output = ()>>>> {
        self.fut.take()
    }
}
