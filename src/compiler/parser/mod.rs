use crate::compiler::ast::*;
use crate::compiler::diagnostic::*;
use crate::compiler::lexer::cursor::Cursor;
use crate::compiler::lexer::token::*;
use crate::compiler::lexer::Lexer;
use crate::source::Span;

mod expr;
mod stmt;

pub struct Parser<'sess> {
    cursor: Cursor<'sess>,
    diagnostics: &'sess mut ErrorReporter,
}

impl<'sess> Parser<'sess> {
    pub fn new(lexer: Lexer<'sess>, diagnostics: &'sess mut ErrorReporter) -> Self {
        Self {
            cursor: lexer.cursor(),
            diagnostics,
        }
    }

    pub fn parse(mut self) -> Module {
        let mut statements = vec![];
        while !self.cursor.eof() {
            statements.push(self.parse_statement());
        }

        Module { statements }
    }

    fn parse_or_recover<T, F: FnMut(&mut Self) -> JlyResult<T>, R: FnMut(&mut Self, Span) -> T>(
        &mut self,
        mut f: F,
        mut recover: R,
    ) -> T {
        let start_span = self.cursor.peek().span;

        match f(self) {
            Ok(result) => result,
            Err(err) => {
                let end_span = self.cursor.prev_span();
                let span = start_span.join(end_span);

                self.diagnostics.report(err.report());
                recover(self, span)
            }
        }
    }

    fn recover_to(&mut self, kind: TokenKind) {
        self.cursor.ignore_while(|kind_actual| kind_actual != kind);
    }

    fn recover_past(&mut self, kind: TokenKind) {
        self.cursor.ignore_while(|kind_actual| kind_actual != kind);
        self.cursor.next();
    }

    fn parse_comma_list<T>(
        &mut self,
        f: fn(&mut Self) -> JlyResult<T>,
        until: TokenKind,
    ) -> JlyResult<Vec<T>> {
        let mut items = vec![f(self)?];
        while self.cursor.eat(punct!(Comma)) && self.cursor.peek().kind != until {
            let item = f(self)?;
            items.push(item);
        }
        Ok(items)
    }

    fn expect(&mut self, kind: TokenKind) -> JlyResult<Token> {
        let token = self.cursor.next();
        if token.kind == kind {
            Ok(token)
        } else {
            Err(Error::UnexpectedToken {
                expected: kind,
                found: token,
            })
        }
    }
}
