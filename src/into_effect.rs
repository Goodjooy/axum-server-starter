use std::{convert::Infallible, error};

use crate::PreparedEffect;

/// fallible prepare effect
pub trait IntoFallibleEffect {
    type Effect: PreparedEffect;
    type Error: std::error::Error;

    fn into_effect(self) -> Result<Self::Effect, Self::Error>;
}

impl<T: PreparedEffect, E: error::Error> IntoFallibleEffect for Result<T, E> {
    type Effect = T;

    type Error = E;

    fn into_effect(self) -> Result<Self::Effect, Self::Error> {
        self
    }
}
impl<T: PreparedEffect> IntoFallibleEffect for T {
    type Effect = T;

    type Error = Infallible;

    fn into_effect(self) -> Result<Self::Effect, Self::Error> {
        Ok(self)
    }
}
