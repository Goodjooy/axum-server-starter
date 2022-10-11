use std::error;

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
