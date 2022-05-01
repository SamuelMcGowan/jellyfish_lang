use std::path::PathBuf;

use crate::compiler::lexer::token::{Token, TokenKind};

#[derive(Debug, Clone)]
pub enum Error {
    ParseError(ParseError),
    FileNotFoundError(PathBuf),
}

#[derive(Default)]
pub struct Diagnostics {
    errors: Vec<Error>,
}

impl Diagnostics {
    pub fn report(&mut self, err: Error) {
        self.errors.push(err);
    }

    pub fn had_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    #[allow(clippy::result_unit_err)]
    pub fn assert_ok(&self) -> Result<(), ()> {
        match self.errors.is_empty() {
            true => Ok(()),
            false => Err(()),
        }
    }

    pub fn print(&self) {
        if self.had_errors() {
            eprintln!("ERRORS WHILE COMPILING:");
            for error in &self.errors {
                // TODO: print errors nicely
                eprintln!("  {:?}", error);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub token: Token,
    pub kind: ParseErrorKind,
}

impl ParseError {
    pub fn new(token: Token, kind: ParseErrorKind) -> Self {
        Self { token, kind }
    }
}

#[derive(Debug, Clone)]
pub enum ParseErrorKind {
    ExpectedKind(TokenKind),
    ExpectedExpr,
    InvalidFieldAccess,
}

pub type ParseResult<T> = Result<T, ParseError>;
