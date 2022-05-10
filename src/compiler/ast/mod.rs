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
    While(WhileLoop),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub ident: Intern<String>,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub ident: Intern<String>,
    pub resolved: Option<VarResolved>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then: Block,
    pub else_: Option<Box<Statement>>,
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expr,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Var(Var),
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
    Neg(Box<Expr>),

    // assignment
    Assignment(Var, Box<Expr>),

    DebugPrint(Box<Expr>),

    DummyExpr,
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub ty: Option<Type>,
}

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        Expr {
            kind,
            span,
            ty: None,
        }
    }
}

macro_rules! expr {
    (boxed $kind:ident ($($arg:expr),+), $span:expr) => {
        Expr::new(ExprKind::$kind ($(Box::new($arg)),*), $span)
    };

    ($kind:ident ($($arg:expr),+), $span:expr) => {
        Expr::new(ExprKind::$kind ($($arg),*), $span)
    };

    ($kind:ident, $span:expr) => {
        Expr::new(ExprKind::$kind, $span)
    };
}

use crate::source::Span;
pub(crate) use expr;
