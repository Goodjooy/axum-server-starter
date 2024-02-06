use std::fmt::{Debug, Display, Formatter};
use std::{any::type_name, error};

use crate::prepare_behave::effect_collectors::state_collector::TypeNotInState;

#[derive(thiserror::Error)]
/// the error while prepare for each [Prepare](crate::Prepare) task
#[error("prepare error on {ty} : {source}")]
pub struct PrepareError {
    ty: &'static str,
    source: Box<dyn error::Error>,
}

impl Debug for PrepareError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <PrepareError as Display>::fmt(self, f)
    }
}

impl PrepareError {
    /// Creates a new [`PrepareError`].
    pub fn new(name: &'static str, src: Box<dyn error::Error>) -> Self {
        Self {
            ty: name,
            source: src,
        }
    }
    pub fn to_prepare_error<P, E: error::Error + 'static>(err: E) -> PrepareError {
        PrepareError::new(type_name::<P>(), Box::new(err))
    }
}

#[derive(Debug, thiserror::Error)]
/// error during the [ServerPrepare::prepare_start](super::ServerPrepare::prepare_start)
pub enum PrepareStartError {
    #[error(transparent)]
    /// prepare error
    Prepare(#[from] PrepareError),
    #[error(transparent)]
    /// state convent error
    State(#[from] TypeNotInState),
    #[error(transparent)]
    IO(#[from] std::io::Error),
}
