use std::fmt::{Display, Formatter};

use crate::compiler::ast::Expr;
use crate::runtime::value::{Object, Value};

use super::*;

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Var(id) => write!(f, "{}", id),
            Self::Value(id) => write!(f, "{}", id),

            // logic
            Self::LogicalOr(a, b) => {
                write!(f, "({} || {})", a, b)
            }
            Self::LogicalAnd(a, b) => {
                write!(f, "({} && {})", a, b)
            }
            Self::LogicalNot(a) => write!(f, "(!{})", a),

            // comparisons
            Self::Equal(a, b) => write!(f, "{} == {}", a, b),
            Self::NotEqual(a, b) => write!(f, "{} != {}", a, b),
            Self::GT(a, b) => write!(f, "({} > {})", a, b),
            Self::LT(a, b) => write!(f, "({} < {})", a, b),
            Self::LTEqual(a, b) => write!(f, "({} <= {})", a, b),
            Self::GTEqual(a, b) => write!(f, "({} >= {})", a, b),

            Self::Add(a, b) => write!(f, "({} + {})", a, b),
            Self::Sub(a, b) => write!(f, "({} - {})", a, b),
            Self::Mul(a, b) => write!(f, "({} * {})", a, b),
            Self::Div(a, b) => write!(f, "({} / {})", a, b),
            Self::Mod(a, b) => write!(f, "({} % {})", a, b),
            Self::Pow(a, b) => write!(f, "({} ^ {})", a, b),

            Self::FieldAccess(a, id) => write!(f, "({}.{})", a, id),
            Self::Call(a, b) => {
                write!(
                    f,
                    "{}({})",
                    a,
                    b.iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Object(obj) => obj.fmt(f),
            Self::String(s) => write!(f, "{:?}", s),
            Self::Integer(n) => write!(f, "{}i", *n),
            Self::Float(n) => write!(f, "{}f", f64::from_bits(*n)),
            Self::Bool(b) => write!(f, "{:?}", b),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::JellyFunction(_) => write!(f, "<function>"),
        }
    }
}
