use std::any::type_name;
use std::future::Future;
use std::{
    convert::Infallible,
    fmt::Debug,
    iter::Cloned,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    time::Duration,
};

use axum::{
    extract::{FromRef, OriginalUri, Path, State},
    routing::get,
    Router,
};

use axum_starter::{
    prepare,
    router::{Fallback, Nest, Route},
    Configure, PrepareDecorator, PrepareError, PrepareMiddlewareEffect, PrepareRouteEffect,
    Provider, ServerPrepare,
};
use axum_starter_macro::FromStateCollector;
use futures::FutureExt;
use tokio::sync::{mpsc, watch};

use futures::future::LocalBoxFuture;
use log::info;
use std::slice::Iter;
use tokio::time::interval;
use tower_http::{metrics::InFlightRequestsLayer, trace::TraceLayer};

/// configure for server starter
#[derive(Debug, Provider, Configure)]
#[conf(
    address(func(path = "|this|this.bar")),
    logger(error = "log::SetLoggerError", func = "simple_logger::init", associate),
    server
)]
#[provider(transparent)]
struct Configure {
    #[provider(r#ref)]
    #[provider(map_to(ty = "&'s str", by = "String::as_str", lifetime = "'s"))]
    #[provider(map_to(ty = "String", by = "Clone::clone"))]
    foo: String,
    bar: SocketAddr,
    #[provider(ignore_global)]
    foo_bar: (i32, i32),
    #[provider(map_to(
        ty = "Cloned<Iter<'a, i32>>",
        by = "|vec|vec.iter().cloned()",
        lifetime = "'a"
    ))]
    iter: Vec<i32>,
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

/// if need ref args ,adding a lifetime
#[prepare(box ShowFoo 'arg)]
fn show_foo<S: AsRef<str> + ?Sized>(f: &S) {
    println!("this is Foo {}", f.as_ref())
}

/// if prepare procedure may occur Error, using `?` after
/// Prepare task Name
#[prepare(Sleeping ?)]
async fn sleep() -> Result<(), Infallible> {
    tokio::time::sleep(Duration::from_secs(2)).await;
    println!("sleep down 2s");
    Ok(())
}

/// prepare support const generic
#[prepare(ShowValue)]
fn the_value<const V: i32>() {
    println!("The value is {}", V)
}

/// using `#[prepare]`
#[prepare(EchoRouter)]
fn echo<S>() -> impl PrepareRouteEffect<S>
where
    S: Clone + Send + Sync + 'static,
{
    Route::new(
        "/:echo",
        get(|Path(echo): Path<String>| async move { format!("Welcome ! {echo}") }),
    )
}

#[prepare(OnFlyRoute)]
fn route<S>() -> impl PrepareRouteEffect<S>
where
    S: Clone + Send + Sync + 'static,
    watch::Receiver<usize>: FromRef<S>,
{
    Route::new(
        "/on-fly",
        get(|State(receive): State<watch::Receiver<usize>>| async move {
            format!("on fly request : {}", *receive.borrow())
        }),
    )
}

#[prepare(box C ?)]
fn routers<S>() -> Result<impl PrepareRouteEffect<S>, Infallible>
where
    S: Clone + Send + Sync + 'static,
{
    Ok((
        Nest::new(
            "/aac/b",
            Router::new().route(
                "/a",
                get(|OriginalUri(uri): OriginalUri| async move { format!("welcome {uri}") }),
            ),
        ),
        Fallback::new(|| async { "oops" }),
    ))
}

pub struct InFlight {
    layer: InFlightRequestsLayer,
    counter: watch::Receiver<usize>,
}

impl<S> PrepareMiddlewareEffect<S> for InFlight {
    type Middleware = InFlightRequestsLayer;

    fn take(self, states: &mut axum_starter::StateCollector) -> Self::Middleware {
        states.insert(self.counter);
        self.layer
    }
}

#[prepare(OnFlyMiddleware)]
fn on_fly_state() -> InFlight {
    let (layer, counter) = InFlightRequestsLayer::pair();
    let (sender, mut receive) = mpsc::channel(1);
    let (sender2, recv2) = watch::channel(0);

    tokio::spawn(async move {
        loop {
            let Some(data) = receive.recv().await else {
                break;
            };

            sender2.send(data).ok();
        }
    });

    tokio::spawn(async move {
        let sender = sender;
        counter
            .run_emitter(Duration::from_millis(500), move |count| {
                let sender = sender.clone();
                async move {
                    sender.send(count).await.ok();
                }
            })
            .await
    });

    InFlight {
        layer,
        counter: recv2,
    }
}

#[prepare(box Show)]
async fn show(FooBar((x, y)): FooBar) {
    println!("the foo bar is local at ({x}, {y})")
}

// following is the post prepare task
async fn print_foo_every_5s(mut request_count : watch::Receiver<usize>){
    let mut interval = interval(Duration::from_secs(5));

    loop {

        interval.tick().await;
        println!("Foo, current request is :{}",*request_count.borrow_and_update());
    }
}

#[tokio::main]
async fn main() {
    start().await
}

async fn start() {
    ServerPrepare::with_config(Configure::new())
        .init_logger()
        .expect("Init Logger Failure")
        .convert_state::<MyState>()
        .prepare_decorator(Decorator)
        .prepare(ShowValue::<_, 11>)
        .prepare_route(C)
        .graceful_shutdown(
            tokio::signal::ctrl_c()
                .map(|_| println!("recv Exit msg"))
                .map(|_| ()),
        )
        .prepare_concurrent(|set| set.join(ShowFoo::<_, String>).join(Show).join(Sleeping))
        .prepare_route(EchoRouter)
        .prepare_route(OnFlyRoute)
        .prepare_middleware::<Route<MyState>, _>(OnFlyMiddleware)
        .layer(TraceLayer::new_for_http())
        .post_prepare(print_foo_every_5s)
        .preparing()
        .await
        .expect("Prepare for starting server failure ")
        .launch()
        .await
        .expect("Server Error")
}

#[derive(Debug, Clone, FromRef, FromStateCollector)]
struct MyState {
    on_fly: watch::Receiver<usize>,
}

#[prepare(sync Decorator)]
fn logger_decorator(addr: SocketAddr) -> LoggerDecorator {
    LoggerDecorator(addr)
}

pub struct LoggerDecorator(SocketAddr);

impl PrepareDecorator for LoggerDecorator {
    type OutFut<Fut, T> = LocalBoxFuture<'static, Result<T, PrepareError>>
        where Fut: Future<Output=Result<T, PrepareError>> + 'static,
              T: 'static;

    fn decorator<Fut, T>(&self, src: &'static str, in_fut: Fut) -> Self::OutFut<Fut, T>
    where
        Fut: Future<Output = Result<T, PrepareError>> + 'static,
        T: 'static,
    {
        Box::pin(async move {
            match in_fut.await {
                Ok(ret) => {
                    info!("prepare[{src}] ret type is {}", type_name::<T>());
                    Ok(ret)
                }
                err @ Err(_) => err,
            }
        })
    }
}
