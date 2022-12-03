use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

pub struct StateCollector(HashMap<TypeId, Box<dyn Any + 'static>>);
