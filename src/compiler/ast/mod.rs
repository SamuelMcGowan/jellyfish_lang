use internment::Intern;

use crate::runtime::value::Value;

pub use self::types::*;

mod fmt;
mod types;

pub struct Module {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub expr: Expr,
}

impl Statement {
    pub fn new(expr: Expr) -> Self {
        Self { expr }
    }
}

#[derive(Debug, Clone)]
pub enum ExprKind {
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

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expr,
    pub then: Expr,
    pub else_: Option<Expr>,
}
