use std::fmt::{Display, Formatter, Result};

use crate::compiler::ast::Expr;

use super::*;

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.kind.fmt(f)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "(")?;
        match self {
            Self::Var(var) => write!(f, "{}", var.ident)?,
            Self::Value(value) => write!(f, "value {}", value.repr())?,

            // logic
            Self::LogicalOr(a, b) => write!(f, "{} || {}", a, b)?,
            Self::LogicalAnd(a, b) => write!(f, "{} && {}", a, b)?,
            Self::LogicalNot(expr) => write!(f, "!{}", expr)?,

            // comparisons
            Self::Equal(a, b) => write!(f, "{} == {}", a, b)?,
            Self::NotEqual(a, b) => write!(f, "{} != {}", a, b)?,
            Self::GT(a, b) => write!(f, "{} > {}", a, b)?,
            Self::LT(a, b) => write!(f, "{} < {}", a, b)?,
            Self::LTEqual(a, b) => write!(f, "{} <= {}", a, b)?,
            Self::GTEqual(a, b) => write!(f, "{} >= {}", a, b)?,

            Self::Add(a, b) => write!(f, "{} + {}", a, b)?,
            Self::Sub(a, b) => write!(f, "{} - {}", a, b)?,
            Self::Mul(a, b) => write!(f, "{} * {}", a, b)?,
            Self::Div(a, b) => write!(f, "{} / {}", a, b)?,
            Self::Mod(a, b) => write!(f, "{} % {}", a, b)?,
            Self::Pow(a, b) => write!(f, "{} ^ {}", a, b)?,
            Self::Neg(expr) => write!(f, "-{}", expr)?,

            // assignment
            Self::Assignment(lhs, rhs) => write!(f, "{} = {}", lhs.ident, rhs)?,

            Self::DebugPrint(expr) => write!(f, "print {}", expr)?,

            Self::DummyExpr => write!(f, "{{err}}")?,
        }
        write!(f, ")")
    }
}

impl Display for Statement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[")?;
        match self {
            Self::Expr(expr) => write!(f, "expr {}", expr)?,
            Self::Block(block) => write!(f, "block {}", block)?,
            Self::VarDecl(var_decl) => write!(f, "var_decl {}", var_decl)?,
            Self::If(if_statement) => write!(f, "if {}", if_statement)?,
            Self::While(while_loop) => write!(f, "while {}", while_loop)?,
        }
        write!(f, "]")
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
        write!(f, "[let {} = {}]", self.ident, self.value)
    }
}

impl Display for IfStatement {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[if {} then {}", self.condition, self.then)?;
        if let Some(else_) = &self.else_ {
            write!(f, " else {}", else_)?;
        }
        write!(f, "]")
    }
}

impl Display for WhileLoop {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "[while {} {}]", self.condition, self.body)
    }
}
