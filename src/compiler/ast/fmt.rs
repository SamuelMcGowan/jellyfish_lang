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
            Self::Var(var) => write!(f, "{}", var.ident),
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

            // assignment
            Self::Assignment(lhs, rhs) => write!(f, "({} = {})", lhs.ident, rhs),

            Self::DebugPrint(expr) => write!(f, "print({})", expr),

            Self::DummyExpr => write!(f, "DummyExpr"),
        }
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::Expr(expr) => write!(f, "Statement({})", expr),
            Self::Block(block) => block.fmt(f),
            Self::VarDecl(var_decl) => var_decl.fmt(f),
            Self::If(if_statement) => if_statement.fmt(f),
            Self::While(while_loop) => while_loop.fmt(f),
        }
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{{ {} }}",
            self.statements
                .iter()
                .map(|stmt| format!("{}", stmt))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Display for VarDecl {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(let {} = {})", self.ident, self.value)
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

impl Display for WhileLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(while {} {})", self.condition, self.body)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Object(obj) => obj.fmt(f),
            Self::String(s) => write!(f, "\"{}\"", s.escape_debug()),
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
