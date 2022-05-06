use internment::Intern;

use crate::fmt::DisplayWithSource;
use crate::source::{Source, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Punctuation(Punctuation),
    Keyword(Keyword),

    Ident(Intern<String>),

    String(Intern<String>),
    Integer(u64),
    Float(u64),
    Bool(bool),

    Error(&'static str),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punctuation {
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    Dot,
    Comma,
    Colon,
    Semicolon,

    Arrow,
    FatArrow,

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    AddEqual,
    SubEqual,
    MulEqual,
    DivEqual,
    ModEqual,
    PowEqual,

    LogicalAnd,
    LogicalOr,
    Bang,

    LT,
    GT,
    LTEqual,
    GTEqual,

    Equal,
    EqualEqual,
    BangEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    DebugPrint,
    If,
    Else,
    Let,
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl DisplayWithSource for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter, context: &Source) -> std::fmt::Result {
        let line_col = context.line_col(self.span.start);
        write!(
            f,
            "Token({}:{}, {:?})",
            line_col.line, line_col.col, self.kind,
        )
    }
}

macro_rules! punct {
    ($p:ident) => {
        TokenKind::Punctuation(Punctuation::$p)
    };
}

macro_rules! kwd {
    ($k:ident) => {
        TokenKind::Keyword(Keyword::$k)
    };
}

pub(crate) use kwd;
pub(crate) use punct;
