use std::pin::Pin;

use futures::Future;

use crate::PreparedEffect;

pub struct SetGraceful(Option<Pin<Box<dyn Future<Output = ()>>>>);

impl SetGraceful {
    pub fn new<F>(future: F) -> Self
    where
        F: Future<Output = ()> + 'static,
    {
        Self(Some(Box::pin(future) as Pin<Box<dyn Future<Output = ()>>>))
    }
}
impl PreparedEffect for SetGraceful {
    fn set_graceful(&mut self) -> Option<std::pin::Pin<Box<dyn Future<Output = ()>>>> {
        self.0.take()
    }
}
