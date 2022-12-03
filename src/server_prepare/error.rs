use std::{any::type_name, error};

use crate::prepare_behave::effect_collectors::state_collector::TypeNotInState;

#[derive(Debug, thiserror::Error)]
/// the error while prepare for each [Prepare](crate::Prepare) task
#[error("prepare error on {ty} : {source}")]
pub struct PrepareError {
    ty: &'static str,
    source: Box<dyn error::Error>,
}

impl PrepareError {
    /// Creates a new [`PrepareError`].
    pub fn new(name: &'static str, src: Box<dyn error::Error>) -> Self {
        Self {
            ty: name,
            source: src,
        }
    }
    pub fn to_prepare_error<P, E: std::error::Error + 'static>(err: E) -> PrepareError {
        PrepareError::new(type_name::<P>(), Box::new(err))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PrepareStartError {
    #[error(transparent)]
    Prepare(#[from] PrepareError),
    #[error(transparent)]
    State(#[from] TypeNotInState),
}
