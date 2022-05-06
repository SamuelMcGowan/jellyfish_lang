use std::fmt::{Display, Formatter, Result};

use crate::compiler::ast::Expr;
use crate::runtime::value::{Object, Value};

use super::*;

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.kind.fmt(f)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
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

            Self::Block(statements) => {
                write!(
                    f,
                    "{{{}}}",
                    statements
                        .iter()
                        .map(|s| format!("{}", s))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }

            Self::IfStatement(if_statement) => write!(f, "{}", if_statement),
            Self::DebugPrint(expr) => write!(f, "print({})", expr),

            Self::DummyExpr => write!(f, "DummyExpr"),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Statement({})", self.expr)
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(if {} then {}", self.condition, self.then)?;
        if let Some(else_) = &self.else_ {
            write!(f, " else {}", else_)?;
        }
        write!(f, ")")
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
            Self::Unit => write!(f, "()"),
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
