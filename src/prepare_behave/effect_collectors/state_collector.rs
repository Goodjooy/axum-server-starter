use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    ops::BitAnd,
};

pub struct StateCollector(HashMap<TypeId, Box<dyn Any + 'static>>);

impl BitAnd for StateCollector {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.into_iter().chain(rhs.0.into_iter()).collect())
    }
}

impl StateCollector {
    pub(crate) fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<T: 'static + Any>(&mut self, data: T) {
        self.0.insert(data.type_id(), Box::new(data));
    }

    pub fn take<T: 'static + Any>(&mut self) -> Result<T, TypeNotInState> {
        self.0
            .remove(&TypeId::of::<T>())
            .map(|data| data.downcast().ok())
            .flatten()
            .map(|data| *data)
            .ok_or(TypeNotInState(type_name::<T>()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Target Type {0} Not in State")]
pub struct TypeNotInState(&'static str);

pub trait FromStateCollector: Sized {
    fn fetch_mut(collector: &mut StateCollector) -> Result<Self, TypeNotInState>;

    fn fetch(mut collector: StateCollector) -> Result<Self, TypeNotInState> {
        FromStateCollector::fetch_mut(&mut collector)
    }
}
