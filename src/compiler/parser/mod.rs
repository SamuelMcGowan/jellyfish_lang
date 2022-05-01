use crate::compiler::ast::*;
use crate::compiler::diagnostic::*;
use crate::compiler::lexer::cursor::Cursor;
use crate::compiler::lexer::Lexer;
use crate::compiler::lexer::token::*;

mod expr;
mod stmt;

pub struct Parser<'sess> {
    cursor: Cursor<'sess>,
    diagnostics: &'sess mut Diagnostics,
}

impl<'sess> Parser<'sess> {
    pub fn new(lexer: Lexer<'sess>, diagnostics: &'sess mut Diagnostics) -> Self {
        Self {
            cursor: lexer.cursor(),
            diagnostics,
        }
    }

    pub fn parse(mut self) -> Module {
        let mut statements = vec![];

        while !self.cursor.eof() {
            let statement = match self.parse_stmt() {
                Ok(stmt) => stmt,
                Err(err) => {
                    self.diagnostics.report(Error::ParseError(err));
                    self.cursor.ignore_while(|kind| kind == punct!(Semicolon));
                    self.cursor.next();
                    continue;
                }
            };
            statements.push(statement);
        }

        Module { statements }
    }

    fn parse_comma_list<T>(
        &mut self,
        f: fn(&mut Self) -> ParseResult<T>,
        until: TokenKind,
    ) -> ParseResult<Vec<T>> {
        let mut items = vec![f(self)?];
        while self.cursor.eat(punct!(Comma)) && self.cursor.peek().kind != until {
            let item = f(self)?;
            items.push(item);
        }
        Ok(items)
    }

    fn expect(&mut self, kind: TokenKind) -> ParseResult<Token> {
        let token = self.cursor.next();
        if token.kind == kind {
            Ok(token)
        } else {
            Err(ParseError::new(token, ParseErrorKind::ExpectedKind(kind)))
        }
    }
}
