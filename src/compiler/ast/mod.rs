use internment::Intern;

use crate::runtime::value::Value;

pub use self::types::*;

mod fmt;
mod types;

pub struct Module {
    pub statements: Vec<Statement>,
}

pub enum Statement {
    DebugPrint(Expr),
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Var(Intern<String>),
    Value(Value),

    // logic
    LogicalOr(Box<Expr>, Box<Expr>),
    LogicalAnd(Box<Expr>, Box<Expr>),
    LogicalNot(Box<Expr>),

    // comparisons
    Equal(Box<Expr>, Box<Expr>),
    NotEqual(Box<Expr>, Box<Expr>),
    LT(Box<Expr>, Box<Expr>),
    GT(Box<Expr>, Box<Expr>),
    LTEqual(Box<Expr>, Box<Expr>),
    GTEqual(Box<Expr>, Box<Expr>),

    // arithmetic
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),

    FieldAccess(Box<Expr>, Intern<String>),
    Call(Box<Expr>, Vec<Expr>),
}
