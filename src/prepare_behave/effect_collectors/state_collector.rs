use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
    ops::BitAnd,
};

/// collect all state during prepare
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

    /// insert a new type into state collect
    ///
    /// if the type previously exist, the new value will overwrite the old one
    pub fn insert<T: 'static + Any>(&mut self, data: T) {
        self.0.insert(data.type_id(), Box::new(data));
    }

    /// take a type from the collector
    ///
    /// if the Value Not exist in collector, it will return [TypeNotInState] Error
    pub fn take<T: 'static + Any>(&mut self) -> Result<T, TypeNotInState> {
        self.0
            .remove(&TypeId::of::<T>())
            .and_then(|data| data.downcast().ok())
            .map(|data| *data)
            .ok_or(TypeNotInState(type_name::<T>()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Target Type {0} Not in State")]
/// then type not in the state collector
pub struct TypeNotInState(&'static str);

/// Mapping type form [StateCollector] to special Type
pub trait FromStateCollector: Sized {
    /// take a part of data
    fn fetch_mut(collector: &mut StateCollector) -> Result<Self, TypeNotInState>;

    /// take data an fetch the ownership
    fn fetch(mut collector: StateCollector) -> Result<Self, TypeNotInState> {
        FromStateCollector::fetch_mut(&mut collector)
    }
}

macro_rules! state_gen {
    ($($id:ident),*$(,)?) => {
        impl<$($id : 'static),*> FromStateCollector for ($($id,)*) {
            #[allow(unused_variables)]
            fn fetch_mut(collector: &mut StateCollector) -> Result<Self, TypeNotInState> {
                Ok((
                    $(
                        collector.take::<$id>()?,
                    )*

                ))
            }
        }

    };
}

state_gen!();
state_gen!(T1);
state_gen!(T1, T2);
state_gen!(T1, T2, T3);
state_gen!(T1, T2, T3, T4);
state_gen!(T1, T2, T3, T4, T5);
state_gen!(T1, T2, T3, T4, T5, T6);
state_gen!(T1, T2, T3, T4, T5, T6, T7);
state_gen!(T1, T2, T3, T4, T5, T6, T7, T8);
