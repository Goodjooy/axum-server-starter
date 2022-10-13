use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use axum::{extract::Path, routing::get, Extension};
use axum_starter::{
    prepare, ExtensionManage, PreparedEffect, Provider, ServeAddress, ServerEffect, ServerPrepare,
};

#[tokio::main]
async fn main() {
    ServerPrepare::with_config(Config {
        id: 11,
        name: "Str".to_string(),
    })
    .append(Student)
    // .append(Echo)
    .prepare_start()
    .await
    .expect("")
    .launch()
    .await
    .expect("");
}


#[prepare(Student 'arg)]
fn arr<'arg>(id: i32, name: & 'arg String) {
    println!("my name is {name} id is {id}")
}

#[prepare(Echo)]
fn adding_echo()->EchoEffect{
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
