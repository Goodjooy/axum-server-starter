# Axum Starter

[![Github](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/Goodjooy/axum-server-starter)
[![Crates.io](https://img.shields.io/crates/v/axum-starter.svg?style=for-the-badge)](https://crates.io/crates/axum-starter)
![License](https://img.shields.io/github/license/Goodjooy/axum-server-starter?style=for-the-badge)

## Why axum-starter

With the growing of the server functions, the code which prepare multiply infrastructures for the server in the main become more and more complex.  
For example, I need connect to `Mysql` and `Redis`, start `MessageQuery` , start GracefulShutdown and so on.  
In other to simplify the start up code with my server project, there comes the `axum-starter`

## Safety

the outer attribute `#![forbid(unsafe_code)]` enable

## Simple Example

The following example using `axum-starter` starting a web server which
server on `http://127.0.0.1:5050`

It can do

1. show info before launch (with `logger` feature)
2. using `simple_logger` and adding TraceLayer as logger middleware
3. request `http://127.0.0.1:5050/greet/{name}` will respond greet with your name

```rust
use axum::{extract::Path, routing::get};
use axum_starter::{prepare, router::Route, ServerPrepare};
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
        .prepare_route(GreetRoute)
        .layer(TraceLayer::new_for_http())
        .no_state()
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
fn greet_route<S, B>() -> Route<S, B>
where
    B: http_body::Body + Send + 'static,
    S: Clone + Send + Sync + 'static,
{
    Route::new("/greet/:name", get(greet))
}
```

## Core Concept

Each task before starting the server call [`Prepare`](https://docs.rs/axum-starter/latest/axum_starter/trait.Prepare.html). Each `Prepare` will Return a `PreparedEffect` for `ServerPrepare` to apply each prepare's effect on the server.
Finally, all `Prepare` are done and the server can be launch

### [`Prepare`](https://docs.rs/axum-starter/latest/axum_starter/trait.Prepare.html) trait

the trait define the prepare task,
after prepare down, it return a `PreparedEffect`

### `PreparedEffect` trait family

the trait family will apply multiply effect on the server. include the following

- [Router](https://docs.rs/axum-starter/latest/axum_starter/trait.PrepareRouteEffect.html)
- [State](https://docs.rs/axum-starter/latest/axum_starter/trait.PrepareStateEffect.html)
- [Middleware](https://docs.rs/axum-starter/latest/axum_starter/trait.PrepareMiddlewareEffect.html)

## `Concurrently` or `Serially`

`Prepare`s will run one by one in default, in another word, they running _serially_,
if you want run some `Prepare`s _concurrently_, you can call [`ServerPrepare::prepare_concurrent`](https://docs.rs/axum-starter/latest/axum_starter/struct.ServerPrepare.html#method.prepare_concurrent), to give a group of `Prepare`s running _concurrently_

## Set Middleware

if you want to adding a middleware on the root of server `Router`, using [`ServerPrepare::layer`](crate::ServerPrepare::layer) then giving the `Layer`

or using [`PrepareMiddlewareEffect`](crate::PrepareMiddlewareEffect) apply middleware in [`Prepare`](crate::Prepare)
