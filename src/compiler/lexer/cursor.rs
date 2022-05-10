use crate::compiler::lexer::token::{Token, TokenKind};
use crate::compiler::lexer::Lexer;
use crate::source::Span;

pub struct Cursor<'sess> {
    lexer: Lexer<'sess>,

    current: Option<Token>,
    prev: Option<Token>,
}

impl<'sess> Cursor<'sess> {
    pub fn new(mut lexer: Lexer<'sess>) -> Self {
        let current = lexer.next();
        Self {
            lexer,

            current,
            prev: None,
        }
    }

    pub fn peek(&self) -> Token {
        match self.current {
            Some(t) => t,
            None => Token::new(TokenKind::Eof, self.lexer.cursor.span()),
        }
    }

    pub fn next(&mut self) -> Token {
        match self.current {
            Some(t) => {
                self.prev = self.current;
                self.current = self.lexer.next();
                t
            }
            None => Token::new(TokenKind::Eof, self.lexer.cursor.span()),
        }
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.matches(kind) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn matches(&mut self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    pub fn ignore_while(&mut self, mut f: impl FnMut(TokenKind) -> bool) {
        while !self.eof() && f(self.peek().kind) {
            self.next();
        }
    }

    pub fn eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    pub fn prev_span(&self) -> Span {
        self.prev.map(|token| token.span).unwrap_or(Span::default())
    }
}
