use std::fmt::{Display, Formatter};

use crate::compiler::lexer::token::*;

use super::Report;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(token: Token, kind: ParseErrorKind) -> Self {
        Self { token, kind }
    }

    pub fn report(&self) -> Report {
        Report {
            title: self.kind.title(),
            msg: format!("{}", self),
            snippet: Some(self.token.span),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ParseErrorKind::ExpectedKind(kind) => write!(
                f,
                "expected token {:?}, found token {:?}",
                kind, self.token.kind
            ),
            ParseErrorKind::ExpectedExpr => {
                write!(f, "expected an expression, found {:?}", self.token.kind)
            }
            ParseErrorKind::InvalidFieldAccess => {
                write!(f, "invalid field access {:?}", self.token.kind)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    ExpectedKind(TokenKind),
    ExpectedExpr,
    InvalidFieldAccess,
}

impl ParseErrorKind {
    pub fn title(&self) -> &'static str {
        match self {
            Self::ExpectedKind(_) => "unexpected token",
            Self::ExpectedExpr => "expected an expression",
            Self::InvalidFieldAccess => "invalid field access",
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;
