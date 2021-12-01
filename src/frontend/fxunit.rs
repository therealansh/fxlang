use crate::frontend::fxfx::FxFx;
use std::cell::RefCell;
use std::rc::Rc;
use crate::frontend::fxclass::{FxClassInstance, FxClass};

#[derive(Debug, Clone)]
pub enum FxUnit {
    Boolean(bool),
    Callable(FxFx),
    Nil,
    Number(f64),
    String(String),
    Instance(Rc<RefCell<FxClassInstance>>),
    Class(Rc<RefCell<FxClass>>)
}

impl FxUnit {
    pub fn equals(&self, other: &FxUnit) -> bool {
        match (self, other) {
            (FxUnit::Nil, FxUnit::Nil) => true,
            (_, FxUnit::Nil) => false,
            (FxUnit::Nil, _) => false,
            (FxUnit::Boolean(left), FxUnit::Boolean(right)) => left == right,
            (FxUnit::Number(left), FxUnit::Number(right)) => left == right,
            (FxUnit::String(left), FxUnit::String(right)) => left == right,
            _ => false //TODO define for class and instance
        }
    }
}