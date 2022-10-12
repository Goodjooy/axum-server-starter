use hyper::server::conn::AddrIncoming;

/// get the address this server are going to bind with
pub trait ServeAddress {
    type Address: Into<std::net::SocketAddr>;
    fn get_address(&self) -> Self::Address;
}

/// change the server configure, this operate can overwrite
/// [PrepareEffect](crate::PreparedEffect)
pub trait ServerEffect {
    fn effect_server(
        &self,
        server: hyper::server::Builder<AddrIncoming>,
    ) -> hyper::server::Builder<AddrIncoming> {
        server
    }
}
