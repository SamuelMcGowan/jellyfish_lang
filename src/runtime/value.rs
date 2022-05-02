use std::fmt::Debug;
use std::rc::Rc;

use internment::Intern;

use super::chunk::Chunk;

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    Object,
    String,
    Integer,
    Float,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Value {
    Object(Rc<Object>),
    String(Intern<String>),
    Integer(u64),
    Float(u64),
    Bool(bool),
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
        }
    }

    pub fn integer(&self) -> u64 {
        match self {
            Self::Integer(n) => *n,
            _ => todo!("need to implement type checking before runtime"),
        }
    }

    pub fn bool(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            _ => todo!("need to implement type checking before runtime"),
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
