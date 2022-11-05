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
