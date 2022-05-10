use internment::Intern;

use crate::source::Cursor;

use self::cursor::Cursor as TokenCursor;
use self::token::*;

pub mod cursor;
pub mod token;

const NUMBER_SEP: char = '_';

pub struct Lexer<'sess> {
    cursor: Cursor<'sess>,
}

impl<'sess> Lexer<'sess> {
    pub fn new(cursor: Cursor<'sess>) -> Self {
        Self { cursor }
    }

    pub fn cursor(self) -> TokenCursor<'sess> {
        TokenCursor::new(self)
    }

    /// Lex a single token.
    pub fn lex_token(&mut self) -> Token {
        loop {
            self.cursor.start_span();

            let c = match self.cursor.advance() {
                Some(c) => c,
                None => return Token::new(TokenKind::Eof, self.cursor.span()),
            };

            let kind = match c {
                ' ' | '\t' | '\r' | '\n' => continue,

                '/' if self.cursor.eat('/') => {
                    self.cursor.eat_while(|c| c != '\n');
                    continue;
                }

                '(' => punct!(LParen),
                ')' => punct!(RParen),
                '{' => punct!(LBrace),
                '}' => punct!(RBrace),
                '[' => punct!(LBracket),
                ']' => punct!(RBracket),

                '.' => punct!(Dot),
                ',' => punct!(Comma),
                ':' => punct!(Colon),
                ';' => punct!(Semicolon),

                '-' if self.cursor.eat('>') => punct!(Arrow),
                '=' if self.cursor.eat('>') => punct!(FatArrow),

                '+' if self.cursor.eat('=') => punct!(AddEqual),
                '-' if self.cursor.eat('=') => punct!(SubEqual),
                '*' if self.cursor.eat('=') => punct!(MulEqual),
                '/' if self.cursor.eat('=') => punct!(DivEqual),
                '%' if self.cursor.eat('=') => punct!(ModEqual),
                '^' if self.cursor.eat('=') => punct!(PowEqual),

                '+' => punct!(Add),
                '-' => punct!(Sub),
                '*' => punct!(Mul),
                '/' => punct!(Div),
                '%' => punct!(Mod),
                '^' => punct!(Pow),

                '>' if self.cursor.eat('=') => punct!(GTEqual),
                '<' if self.cursor.eat('=') => punct!(LTEqual),
                '>' => punct!(GT),
                '<' => punct!(LT),

                '=' if self.cursor.eat('=') => punct!(EqualEqual),
                '!' if self.cursor.eat('=') => punct!(BangEqual),
                '=' => punct!(Equal),

                '&' if self.cursor.eat('&') => punct!(LogicalAnd),
                '|' if self.cursor.eat('|') => punct!(LogicalOr),
                '!' => punct!(Bang),

                'a'..='z' | 'A'..='Z' | '_' => self.lex_alpha(),
                c @ '0'..='9' => self.lex_number(c),

                '"' => self.lex_string(),

                _ => TokenKind::Error("Unexpected character."),
            };

            return Token::new(kind, self.cursor.span());
        }
    }

    /// Lex an alphanumeric keyword or identifier.
    ///
    /// Expects the first character (which should be `a-zA-Z_`) to have
    /// been consumed.
    fn lex_alpha(&mut self) -> TokenKind {
        while let 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' = self.cursor.peek() {
            self.cursor.advance();
        }

        match self.cursor.lexeme() {
            "print" => kwd!(DebugPrint),

            "if" => kwd!(If),
            "else" => kwd!(Else),
            "while" => kwd!(While),

            "let" => kwd!(Let),

            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),

            s => TokenKind::Ident(Intern::new(s.to_string())),
        }
    }

    /// Lex a string.
    ///
    /// Expects the first `"` to have been consumed.
    fn lex_string(&mut self) -> TokenKind {
        let mut s = String::new();
        loop {
            let c = match self.cursor.advance() {
                Some(c) => c,
                None => return TokenKind::Error("Unterminated string."),
            };

            let d = match c {
                '\\' => match self.lex_escape() {
                    Ok(escape) => escape,
                    Err(msg) => return TokenKind::Error(msg),
                },
                '"' => break,
                d => d as char,
            };

            s.push(d);
        }

        TokenKind::String(Intern::new(s))
    }

    /// Lex an escape sequence.
    ///
    /// Expects the backslash to already have been consumed.
    fn lex_escape(&mut self) -> Result<char, &'static str> {
        let c = self
            .cursor
            .advance()
            .ok_or("Unexpected end of input in escape sequence.")?;

        Ok(match c {
            't' => '\t',
            'r' => '\r',
            'n' => '\n',
            c => c as char,
        })
    }

    /// Lex a number.
    fn lex_number(&mut self, first: char) -> TokenKind {
        let second = self.cursor.peek();
        match first {
            '0' if self.cursor.eat('x') => self.lex_integer_with_radix(16),
            '0' if self.cursor.eat('b') => self.lex_integer_with_radix(2),
            '0' if self.cursor.eat('o') => self.lex_integer_with_radix(8),

            // `0_` is illegal, but we should lex it anyway.
            '0' if second.is_digit(10) || second == NUMBER_SEP => {
                self.collect_digits(10);
                self.eat_fractional_part(10);
                TokenKind::Error("leading zeroes")
            }
            n => self.lex_decimal(n),
        }
    }

    /// Lex a decimal number that may be an integer or a float.
    fn lex_decimal(&mut self, first: char) -> TokenKind {
        let mut integer = first.to_string();
        integer.push_str(&self.collect_digits(10));

        if self.cursor.eat('.') {
            let fraction = self.collect_digits(10);
            if fraction.is_empty() {
                return TokenKind::Error("fractional part is empty");
            }

            let exponent = if self.cursor.eat('e') | self.cursor.eat('E') {
                let sign = if self.cursor.eat('-') {
                    -1
                } else {
                    self.cursor.eat('+');
                    1
                };

                let digits = self.collect_digits(10);
                let exponent = match parse_digits(digits, 10) {
                    Some(n) => n,
                    None => return TokenKind::Error("exponent overflowed"),
                };

                sign * exponent as i32
            } else {
                0
            };

            let f: f64 = minimal_lexical::parse_float(
                integer.as_bytes().iter(),
                fraction.as_bytes().iter(),
                exponent,
            );

            TokenKind::Float(f.to_bits())
        } else {
            let n = match parse_digits(integer, 10) {
                Some(n) => n,
                None => return TokenKind::Error("integer literal overflowed"),
            };
            TokenKind::Integer(n)
        }
    }

    /// Lex a fraction-less integer of the given radix.
    fn lex_integer_with_radix(&mut self, radix: u32) -> TokenKind {
        let integer = self.collect_digits(radix);
        if integer.is_empty() {
            return TokenKind::Error("empty integer literal");
        }

        if self.eat_fractional_part(radix) {
            return TokenKind::Error("only decimal numbers may have a fractional part");
        }

        let n = match parse_digits(integer, radix) {
            Some(n) => n,
            None => return TokenKind::Error("integer literal overflowed"),
        };

        TokenKind::Integer(n)
    }

    /// Eat all digits of a number with the given radix, skip any
    /// separators, and return the digits as a (seperator-less) string.
    fn collect_digits(&mut self, radix: u32) -> String {
        let mut digits = String::new();
        loop {
            match self.cursor.peek() {
                c if c == NUMBER_SEP => (),
                c if c.is_digit(radix) => digits.push(c),
                _ => return digits,
            };
            self.cursor.advance();
        }
    }

    /// Eat the fractional part of a number if possible, and return
    /// whether anything was eaten.
    fn eat_fractional_part(&mut self, radix: u32) -> bool {
        if self.cursor.eat('.') {
            if self.cursor.eat('e') | self.cursor.eat('E') {
                self.cursor.eat('+');
                self.cursor.eat('-');
                self.collect_digits(radix);
            }
            true
        } else {
            false
        }
    }
}

impl<'sess> Iterator for Lexer<'sess> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.lex_token())
    }
}

fn parse_digits(digits: String, radix: u32) -> Option<u64> {
    let mut n: u64 = 0;
    for digit in digits.chars() {
        let digit = digit.to_digit(radix).unwrap();
        n = n
            .checked_mul(radix as u64)
            .and_then(|n| n.checked_add(digit as u64))?;
    }
    Some(n)
}
