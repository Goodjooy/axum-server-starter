

/// Prepare for Global State
///
/// for instance the Connection Pool of Database
/// prepare side effect of [PrepareState]
pub trait PrepareStateEffect:'static {
    type StateType: 'static;

    fn take_state(self) -> Self::StateType;
}
