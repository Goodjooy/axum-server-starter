pub trait ServeBind {
    type Address: Into<std::net::SocketAddr>;

    fn get_address(&self) -> Self::Address;
}

pub trait ServerEffect {
    fn effect_server<I>(&self, server: hyper::server::Builder<I>) -> hyper::server::Builder<I>;
}
