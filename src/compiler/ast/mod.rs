use internment::Intern;

use crate::compiler::passes::resolve::VarResolved;
use crate::runtime::value::Value;

pub use self::types::*;

mod fmt;
mod types;

pub struct Module {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub num_vars: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expr(Expr),
    Block(Block),
    VarDecl(VarDecl),
    If(IfStatement),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub ident: Intern<String>,
    pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then: Block,
    pub else_: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Var(Intern<String>),
    VarResolved(VarResolved),
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

    DebugPrint(Box<Expr>),

    DummyExpr,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Option<Type>,
}

impl Expr {
    pub fn new(kind: ExprKind) -> Self {
        Expr { kind, ty: None }
    }
}

macro_rules! expr {
    (boxed $kind:ident ($($arg:expr),+)) => {
        Expr::new(ExprKind::$kind ($(Box::new($arg)),*))
    };

    ($kind:ident ($($arg:expr),+)) => {
        Expr::new(ExprKind::$kind ($($arg),*))
    };

    ($kind:ident) => {
        Expr::new(ExprKind::$kind)
    };
}

pub(crate) use expr;
