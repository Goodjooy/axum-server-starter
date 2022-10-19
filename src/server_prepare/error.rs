use std::{any::type_name, error};

#[derive(Debug, thiserror::Error)]
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
}
pub fn to_prepare_error<P, E: std::error::Error + 'static>(err: E) -> PrepareError {
    PrepareError::new(type_name::<P>(), Box::new(err))
}

pub fn flatten_result<T1, T2, E>((l, r): (Result<T1, E>, Result<T2, E>)) -> Result<(T1, T2), E> {
    Ok((l?, r?))
}
