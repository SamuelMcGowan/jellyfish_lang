use internment::Intern;

use crate::runtime::value::Value;

pub use self::types::*;

mod fmt;
mod types;

pub struct Module {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    If(Box<IfStatement>),
    DebugPrint(Expr),
    ExprStmt(Expr),
    DummyStmt,
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

    Block(Vec<Statement>),
    IfStatement(Box<IfStatement>),

    DummyExpr,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then: Expr,
    pub else_: Option<Expr>,
}
