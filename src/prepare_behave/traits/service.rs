use hyper::server::{Builder, accept::Accept};

/// [Prepare] effect may change the server configure
pub trait ServerEffect: Sized {
    type Accept:Accept;
    /// changing the serve config
    ///
    /// ## Note
    /// the changing of server config might be overwrite by `Config` using [crate::ServerEffect]
    fn config_serve(self, server: Builder<Self::Accept>) -> Builder<Self::Accept> {
        server
    }
}

