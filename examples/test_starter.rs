use std::net::{Ipv4Addr, SocketAddr};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use axum::extract::{FromRef, State};
use axum_starter::state::AddState;
use axum_starter::{
    prepare, Configure, FromStateCollector, PrepareStateEffect, Provider, ServerPrepare,
};
use http::Request;
use tower::{Service, ServiceExt};
use tower_http::trace::TraceLayer;

fn main() {
    let rt = tokio::runtime::Runtime::new().expect("init rt failure");
    rt.block_on(task())
}

async fn task() {
    let mut service = ServerPrepare::test_with_config(Configure::default())
        .init_logger()
        .expect("init Logger Failure")
        .prepare_state(AddCounter)
        .convert_state::<TestState>()
        .layer(TraceLayer::new_for_http())
        .preparing_test(handle)
        .await
        .expect("prepare for service failure");

    let request = Request::builder()
        .body(axum::body::Body::empty())
        .expect("generate Request Body Error");

    let resp = service.call(request).await.expect("Response Error");
    let body = resp.plain().await.unwrap();

    println!("Resp is :{body}",);
    assert_eq!(body, "current Count is 2");

    service.ready().await.expect("Waiting for Ready");

    let request = Request::builder()
        .body(axum::body::Body::empty())
        .expect("generate Request Body Error");

    let resp = service.call(request).await.expect("Response Error");
    let body = resp.plain().await.unwrap();

    println!("Resp is :{body}",);
    assert_eq!(body, "current Count is 3");
}

/// configure for server starter
#[derive(Debug, Provider, Configure, Default)]
#[conf(
    logger(error = "log::SetLoggerError", func = "simple_logger::init", associate),
    server
)]
#[provider(transparent)]
struct Configure {}

#[derive(Debug, FromRef, Clone, FromStateCollector)]
struct TestState {
    count: Arc<AtomicI32>,
}

async fn handle(State(counter): State<Arc<AtomicI32>>) -> String {
    let num = counter.fetch_add(1, Ordering::Relaxed);
    format!("current Count is {num}")
}

// prepare start

#[prepare(AddCounter)]
fn prepare_counter() -> impl PrepareStateEffect {
    AddState(Arc::new(AtomicI32::new(2)))
}
