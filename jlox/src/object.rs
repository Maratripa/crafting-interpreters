use crate::{
    class::{Class, Instance},
    functions::Callable,
};

use std::{cell::RefCell, fmt::Display, rc::Rc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Cast conversion failed: {value} is not a number")]
    NaN { value: String },
}

#[derive(Debug, Default)]
pub enum Object {
    #[default]
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Function(Rc<dyn Callable<E = crate::interpreter::Error>>),
    Class(Rc<RefCell<Class>>),
    Instance(Rc<RefCell<Instance>>),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Function(func) => write!(f, "{:?}", func),
            Self::Class(klass) => write!(f, "{}", klass.borrow().to_string()),
            Self::Instance(inst) => write!(f, "{}", inst.borrow().to_string()),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Nil, Self::Nil) => true,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => false,
        }
    }
}
