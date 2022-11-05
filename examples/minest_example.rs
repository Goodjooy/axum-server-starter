use axum::{extract::Path, routing::get};
use axum_starter::{prepare, router::Route, PreparedEffect, ServerPrepare};
use config::Conf;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    start().await;
}

async fn start() {
    ServerPrepare::with_config(Conf::default())
        .init_logger()
        .expect("Init Logger Error")
        .append(GreetRoute)
        .with_global_middleware(TraceLayer::new_for_http())
        .prepare_start()
        .await
        .expect("Prepare for Start Error")
        .launch()
        .await
        .expect("Server Error");
}

mod config {
    use std::net::Ipv4Addr;

    use axum_starter::{Configure, Provider};
    use log::LevelFilter;
    use log::SetLoggerError;
    use simple_logger::SimpleLogger;

    // prepare the init configure
    #[derive(Debug, Default, Provider, Configure)]
    #[conf(
        address(func(
            path = "||(Ipv4Addr::LOCALHOST, 5050)",
            ty = "(Ipv4Addr, u16)",
            associate,
        )),
        logger(
            func = "||SimpleLogger::new().with_level(LevelFilter::Debug).init()",
            error = "SetLoggerError",
            associate,
        ),
        server
    )]
    pub(super) struct Conf {}
}

async fn greet(Path(name): Path<String>) -> String {
    format!("Welcome {name} !")
}

#[prepare(GreetRoute)]
fn greet_route() -> impl PreparedEffect {
    Route::new("/greet/:name", get(greet))
}
