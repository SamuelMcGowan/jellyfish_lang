pub mod repr;

use internment::Intern;

use std::fmt::Debug;
use std::rc::Rc;

use crate::runtime::vm::RuntimeError;

use super::chunk::Chunk;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Object,
    String,
    Integer,
    Float,
    Bool,
    Unit,
}

#[derive(Debug, Clone)]
pub enum Value {
    Object(Rc<Object>),
    String(Intern<String>),
    Integer(i64),
    Float(u64),
    Bool(bool),
    Unit,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Object(a), Self::Object(b)) => Rc::ptr_eq(a, b),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Value {
    pub fn ty(&self) -> Type {
        match self {
            Self::Object(_) => Type::Object,
            Self::String(_) => Type::String,
            Self::Integer(_) => Type::Integer,
            Self::Float(_) => Type::Float,
            Self::Bool(_) => Type::Bool,
            Self::Unit => Type::Unit,
        }
    }

    pub fn integer(&self) -> Result<i64, RuntimeError> {
        match self {
            Self::Integer(n) => Ok(*n),
            other => Err(RuntimeError::TypeError {
                expected: Type::Integer,
                found: other.ty(),
            }),
        }
    }

    pub fn bool(&self) -> Result<bool, RuntimeError> {
        match self {
            Self::Bool(b) => Ok(*b),
            other => Err(RuntimeError::TypeError {
                expected: Type::Bool,
                found: other.ty(),
            }),
        }
    }
}

#[derive(Debug)]
pub enum Object {
    JellyFunction(Box<JellyFunction>),
}

#[derive(Debug)]
pub struct JellyFunction {
    pub chunk: Chunk,
    pub arity: usize,
    pub return_type: Type,
}
