use tower::Layer;

/// prepare for Middleware
///
/// it can adding middleware and state

pub trait MiddlewarePrepareEffect: Sized + 'static {
    type Middleware<S>: Layer<S> + 'static;

    type StateType: 'static;

    fn take<S>(self) -> (Self::Middleware<S>, Self::StateType);
}
