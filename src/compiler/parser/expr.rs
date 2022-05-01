use crate::runtime::value::*;

use super::*;

/// Precedence of an infix operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prec {
    LogicalOr,
    LogicalAnd,
    LogicalNot,

    Comparison,

    Term,
    Factor,
    Exponent,
    Negative,

    Call,
}

/// Associativity of an infix operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

struct PrefixFunction<'sess>(fn(&mut Parser<'sess>, TokenKind) -> ParseResult<Expr>);

impl<'sess> PrefixFunction<'sess> {
    fn from(kind: TokenKind) -> Option<Self> {
        Some(match kind {
            TokenKind::Ident(_) => Self(Parser::parse_var),
            TokenKind::String(_) => Self(Parser::parse_string),
            TokenKind::Integer(_) => Self(Parser::parse_integer),
            TokenKind::Float(_) => Self(Parser::parse_float),
            TokenKind::Bool(_) => Self(Parser::parse_bool),
            punct!(Bang) => Self(Parser::parse_logical_not),
            punct!(Sub) => Self(Parser::parse_negative),
            punct!(LParen) => Self(Parser::parse_grouping),
            _ => return None,
        })
    }
}

enum InfixRuleKind<'sess> {
    Basic,
    Func(fn(&mut Parser<'sess>, Expr) -> ParseResult<Expr>),
}

struct InfixRule<'sess> {
    prec: usize,
    assoc: Assoc,
    kind: InfixRuleKind<'sess>,
}

impl<'sess> InfixRule<'sess> {
    fn new(prec: usize, assoc: Assoc, kind: InfixRuleKind<'sess>) -> Self {
        Self { prec, assoc, kind }
    }

    fn from(kind: TokenKind) -> Option<Self> {
        use InfixRuleKind::*;

        macro_rules! rule {
            ($kind:expr, $prec:ident) => {
                Self::new(Prec::$prec as usize, Assoc::Left, $kind)
            };
            ($kind:expr, $prec:ident, $assoc:ident) => {
                Self::new(Prec::$prec as usize, Assoc::$assoc, $kind)
            };
        }

        Some(match kind {
            // logic
            punct!(LogicalOr) => rule!(Basic, LogicalOr),
            punct!(LogicalAnd) => rule!(Basic, LogicalAnd),

            // comparisons
            punct!(EqualEqual)
            | punct!(BangEqual)
            | punct!(LT)
            | punct!(GT)
            | punct!(LTEqual)
            | punct!(GTEqual) => rule!(Basic, Comparison),

            // arithmetic
            punct!(Add) | punct!(Sub) => rule!(Basic, Term),
            punct!(Mul) | punct!(Div) | punct!(Mod) => rule!(Basic, Factor),
            punct!(Pow) => rule!(Basic, Exponent, Right),

            punct!(Dot) => rule!(Basic, Call),

            punct!(LParen) => rule!(Func(Parser::parse_call), Call),

            _ => return None,
        })
    }
}

impl<'sess> Parser<'sess> {
    pub fn parse_expr(&mut self) -> ParseResult<Expr> {
        self.parse_prec(0)
    }

    fn parse_prec(&mut self, min_prec: usize) -> ParseResult<Expr> {
        let lhs_token = self.cursor.next();

        let prefix_fn = PrefixFunction::from(lhs_token.kind)
            .ok_or_else(|| ParseError::new(lhs_token, ParseErrorKind::ExpectedExpr))?;

        // Parse the prefix, which is either a prefix operator or a value.
        let mut expr = prefix_fn.0(self, lhs_token.kind)?;

        // While there is an infix operator that is part of the same expression,
        // parse infixed expressions.
        while let Some(rule) = InfixRule::from(self.cursor.peek().kind) {
            if rule.prec < min_prec {
                break;
            }

            expr = match rule.kind {
                InfixRuleKind::Basic => {
                    let op = self.cursor.next().kind;
                    let rhs_token = self.cursor.peek();

                    let min_prec = if rule.assoc == Assoc::Left {
                        rule.prec + 1
                    } else {
                        rule.prec
                    };

                    let lhs = expr;

                    let rhs = self.parse_prec(min_prec)?;

                    self.binary_op(op, lhs, rhs, lhs_token, rhs_token)?
                }
                InfixRuleKind::Func(f) => f(self, expr)?,
            };
        }

        Ok(expr)
    }

    fn binary_op(
        &mut self,
        op: TokenKind,
        lhs: Expr,
        rhs: Expr,
        _lhs_token: Token,
        rhs_token: Token,
    ) -> ParseResult<Expr> {
        let lhs = Box::new(lhs);
        let rhs = Box::new(rhs);

        // NOTE: must be kept in sync with the infix rules.
        Ok(match op {
            // logic
            punct!(LogicalOr) => Expr::LogicalOr(lhs, rhs),
            punct!(LogicalAnd) => Expr::LogicalAnd(lhs, rhs),

            // comparisons
            punct!(EqualEqual) => Expr::Equal(lhs, rhs),
            punct!(BangEqual) => Expr::NotEqual(lhs, rhs),
            punct!(LT) => Expr::LT(lhs, rhs),
            punct!(GT) => Expr::GT(lhs, rhs),
            punct!(LTEqual) => Expr::LTEqual(lhs, rhs),
            punct!(GTEqual) => Expr::GTEqual(lhs, rhs),

            // arithmetic
            punct!(Add) => Expr::Add(lhs, rhs),
            punct!(Sub) => Expr::Sub(lhs, rhs),
            punct!(Mul) => Expr::Mul(lhs, rhs),
            punct!(Div) => Expr::Div(lhs, rhs),
            punct!(Mod) => Expr::Mod(lhs, rhs),
            punct!(Pow) => Expr::Pow(lhs, rhs),

            punct!(Dot) => match *rhs {
                Expr::Var(id) => Expr::FieldAccess(lhs, id),
                _ => {
                    return Err(ParseError::new(
                        rhs_token,
                        ParseErrorKind::InvalidFieldAccess,
                    ));
                }
            },

            _ => unreachable!(),
        })
    }

    fn parse_call(&mut self, expr: Expr) -> ParseResult<Expr> {
        self.expect(punct!(LParen))?;

        let args = if self.cursor.matches(punct!(RParen)) {
            vec![]
        } else {
            self.parse_comma_list(Self::parse_expr, punct!(RParen))?
        };

        self.expect(punct!(RParen))?;

        Ok(Expr::Call(Box::new(expr), args))
    }

    fn parse_var(&mut self, token: TokenKind) -> ParseResult<Expr> {
        match token {
            TokenKind::Ident(id) => Ok(Expr::Var(id)),
            _ => unreachable!(),
        }
    }

    fn parse_string(&mut self, token: TokenKind) -> ParseResult<Expr> {
        match token {
            TokenKind::String(id) => Ok(Expr::Value(Value::String(id))),
            _ => unreachable!(),
        }
    }

    fn parse_integer(&mut self, token: TokenKind) -> ParseResult<Expr> {
        match token {
            TokenKind::Integer(n) => Ok(Expr::Value(Value::Integer(n))),
            _ => unreachable!(),
        }
    }

    fn parse_float(&mut self, token: TokenKind) -> ParseResult<Expr> {
        match token {
            TokenKind::Float(f) => Ok(Expr::Value(Value::Float(f))),
            _ => unreachable!(),
        }
    }

    fn parse_bool(&mut self, token: TokenKind) -> ParseResult<Expr> {
        match token {
            TokenKind::Bool(b) => Ok(Expr::Value(Value::Bool(b))),
            _ => unreachable!(),
        }
    }

    fn parse_logical_not(&mut self, _token: TokenKind) -> ParseResult<Expr> {
        let expr = self.parse_prec(Prec::LogicalNot as usize)?;
        Ok(Expr::LogicalNot(Box::new(expr)))
    }

    fn parse_negative(&mut self, _token: TokenKind) -> ParseResult<Expr> {
        let expr = self.parse_prec(Prec::Negative as usize + 1)?;
        Ok(Expr::LogicalNot(Box::new(expr)))
    }

    fn parse_grouping(&mut self, _token: TokenKind) -> ParseResult<Expr> {
        let expr = self.parse_expr()?;
        self.expect(punct!(RParen))?;
        Ok(expr)
    }
}
