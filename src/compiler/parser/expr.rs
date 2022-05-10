use crate::runtime::value::*;

use super::*;

/// Precedence of an infix operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Prec {
    Assignment,

    LogicalOr,
    LogicalAnd,
    LogicalNot,

    Comparison,

    Term,
    Factor,
    Exponent,
    Negative,
}

/// Associativity of an infix operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

struct PrefixFunction<'sess>(fn(&mut Parser<'sess>, Token) -> JlyResult<Expr>);

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
            kwd!(DebugPrint) => Self(Parser::parse_print),
            _ => return None,
        })
    }
}

enum InfixRuleKind<'sess> {
    Basic,
    Func(fn(&mut Parser<'sess>, Expr) -> JlyResult<Expr>),
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

            // assignment
            punct!(Equal) => rule!(Func(Parser::parse_assignment), Assignment),

            _ => return None,
        })
    }
}

impl<'sess> Parser<'sess> {
    pub fn parse_expr(&mut self) -> JlyResult<Expr> {
        self.parse_prec(0)
    }

    fn parse_prec(&mut self, min_prec: usize) -> JlyResult<Expr> {
        let lhs_token = self.cursor.next();

        let prefix_fn =
            PrefixFunction::from(lhs_token.kind).ok_or(Error::ExpectedExpression(lhs_token))?;

        // Parse the prefix, which is either a prefix operator or a value.
        let mut expr = prefix_fn.0(self, lhs_token)?;

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
        lhs_token: Token,
        rhs_token: Token,
    ) -> JlyResult<Expr> {
        let span = lhs_token.span.join(rhs_token.span);

        // NOTE: must be kept in sync with the infix rules.
        Ok(match op {
            // logic
            punct!(LogicalOr) => expr!(boxed LogicalOr(lhs, rhs), span),
            punct!(LogicalAnd) => expr!(boxed LogicalAnd(lhs, rhs), span),

            // comparisons
            punct!(EqualEqual) => expr!(boxed Equal(lhs, rhs), span),
            punct!(BangEqual) => expr!(boxed NotEqual(lhs, rhs), span),
            punct!(LT) => expr!(boxed LT(lhs, rhs), span),
            punct!(GT) => expr!(boxed GT(lhs, rhs), span),
            punct!(LTEqual) => expr!(boxed LTEqual(lhs, rhs), span),
            punct!(GTEqual) => expr!(boxed GTEqual(lhs, rhs), span),

            // arithmetic
            punct!(Add) => expr!(boxed Add(lhs, rhs), span),
            punct!(Sub) => expr!(boxed Sub(lhs, rhs), span),
            punct!(Mul) => expr!(boxed Mul(lhs, rhs), span),
            punct!(Div) => expr!(boxed Div(lhs, rhs), span),
            punct!(Mod) => expr!(boxed Mod(lhs, rhs), span),
            punct!(Pow) => expr!(boxed Pow(lhs, rhs), span),

            _ => unreachable!(),
        })
    }

    fn parse_assignment(&mut self, lhs: Expr) -> JlyResult<Expr> {
        // this should be true, but just in case the function is called from
        // somewhere else..
        self.expect(punct!(Equal))?;

        let rhs = self.parse_prec(Prec::Assignment as usize)?;

        let span = lhs.span.join(rhs.span);

        let lhs = match lhs.kind {
            ExprKind::Var(var) => var,
            _ => return Err(Error::InvalidAssignmentTarget(lhs)),
        };

        Ok(expr!(Assignment(lhs, Box::new(rhs)), span))
    }

    fn parse_print(&mut self, print_token: Token) -> JlyResult<Expr> {
        let lparen = self.expect(punct!(LParen))?;
        let expr = self.parse_grouping(lparen)?;

        let span = print_token.span.join(expr.span);

        Ok(expr!(boxed DebugPrint(expr), span))
    }

    fn parse_var(&mut self, token: Token) -> JlyResult<Expr> {
        match token.kind {
            TokenKind::Ident(ident) => Ok(expr!(
                Var(Var {
                    ident,
                    resolved: None
                }),
                token.span
            )),
            _ => unreachable!(),
        }
    }

    fn parse_string(&mut self, token: Token) -> JlyResult<Expr> {
        match token.kind {
            TokenKind::String(id) => Ok(expr!(Value(Value::String(id)), token.span)),
            _ => unreachable!(),
        }
    }

    fn parse_integer(&mut self, token: Token) -> JlyResult<Expr> {
        match token.kind {
            TokenKind::Integer(n) => Ok(expr!(Value(Value::Integer(n as i64)), token.span)),
            _ => unreachable!(),
        }
    }

    fn parse_float(&mut self, token: Token) -> JlyResult<Expr> {
        match token.kind {
            TokenKind::Float(f) => Ok(expr!(Value(Value::Float(f)), token.span)),
            _ => unreachable!(),
        }
    }

    fn parse_bool(&mut self, token: Token) -> JlyResult<Expr> {
        match token.kind {
            TokenKind::Bool(b) => Ok(expr!(Value(Value::Bool(b)), token.span)),
            _ => unreachable!(),
        }
    }

    fn parse_logical_not(&mut self, token: Token) -> JlyResult<Expr> {
        let expr = self.parse_prec(Prec::LogicalNot as usize)?;
        let span = token.span.join(expr.span);
        Ok(expr!(LogicalNot(Box::new(expr)), span))
    }

    fn parse_negative(&mut self, token: Token) -> JlyResult<Expr> {
        let expr = self.parse_prec(Prec::Negative as usize + 1)?;
        let span = token.span.join(expr.span);
        Ok(expr!(Neg(Box::new(expr)), span))
    }

    fn parse_grouping(&mut self, _token: Token) -> JlyResult<Expr> {
        let expr = self.parse_or_recover(Self::parse_expr, |s, span| {
            s.recover_to(punct!(RParen));
            expr!(DummyExpr, span)
        });
        self.expect(punct!(RParen))?;
        Ok(expr)
    }
}
