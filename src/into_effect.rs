use crate::PreparedEffect;


pub trait IntoFallibleEffect {
    type Effect:PreparedEffect;
    type Error:std::error::Error;

    fn play_effect(self)->Result<Self::Effect,Self::Error>;
}