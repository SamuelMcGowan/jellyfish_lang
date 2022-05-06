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
}

/// Associativity of an infix operator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Assoc {
    Left,
    Right,
}

struct PrefixFunction<'sess>(fn(&mut Parser<'sess>, TokenKind) -> JlyResult<Expr>);

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
            punct!(LBrace) => Self(Parser::parse_block_expr),
            kwd!(If) => Self(Parser::parse_if_expr),
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
        _rhs_token: Token,
    ) -> JlyResult<Expr> {
        // NOTE: must be kept in sync with the infix rules.
        Ok(match op {
            // logic
            punct!(LogicalOr) => expr!(boxed LogicalOr(lhs, rhs)),
            punct!(LogicalAnd) => expr!(boxed LogicalAnd(lhs, rhs)),

            // comparisons
            punct!(EqualEqual) => expr!(boxed Equal(lhs, rhs)),
            punct!(BangEqual) => expr!(boxed NotEqual(lhs, rhs)),
            punct!(LT) => expr!(boxed LT(lhs, rhs)),
            punct!(GT) => expr!(boxed GT(lhs, rhs)),
            punct!(LTEqual) => expr!(boxed LTEqual(lhs, rhs)),
            punct!(GTEqual) => expr!(boxed GTEqual(lhs, rhs)),

            // arithmetic
            punct!(Add) => expr!(boxed Add(lhs, rhs)),
            punct!(Sub) => expr!(boxed Sub(lhs, rhs)),
            punct!(Mul) => expr!(boxed Mul(lhs, rhs)),
            punct!(Div) => expr!(boxed Div(lhs, rhs)),
            punct!(Mod) => expr!(boxed Mod(lhs, rhs)),
            punct!(Pow) => expr!(boxed Pow(lhs, rhs)),

            _ => unreachable!(),
        })
    }

    fn parse_block_expr(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        Ok(expr!(Block(self.parse_block()?)))
    }

    pub fn parse_if_expr(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        let condition = self.parse_expr()?;

        let then = self.parse_if_arm()?;

        let else_ = if self.cursor.eat(kwd!(Else)) {
            Some(if self.cursor.matches(kwd!(If)) {
                self.parse_expr()?
            } else {
                self.parse_if_arm()?
            })
        } else {
            None
        };

        Ok(expr!(boxed If(IfExpr {
            condition,
            then,
            else_,
        })))
    }

    fn parse_if_arm(&mut self) -> JlyResult<Expr> {
        let token = self.cursor.peek();
        if token.kind == punct!(LBrace) || token.kind == punct!(LParen) {
            self.parse_expr()
        } else {
            Err(Error::ExpectedIfArm(self.cursor.next()))
        }
    }

    fn parse_block(&mut self) -> JlyResult<Vec<Statement>> {
        let mut statements = vec![];

        while !(self.cursor.eof() || self.cursor.matches(punct!(RBrace))) {
            statements.push(self.parse_statement());
        }
        self.expect(punct!(RBrace))?;

        Ok(statements)
    }

    fn parse_print(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        let lparen = self.expect(punct!(LParen))?.kind;
        let expr = self.parse_grouping(lparen)?;
        Ok(expr!(boxed DebugPrint(expr)))
    }

    fn parse_var(&mut self, token: TokenKind) -> JlyResult<Expr> {
        match token {
            TokenKind::Ident(id) => Ok(Expr::new(ExprKind::Var(id))),
            _ => unreachable!(),
        }
    }

    fn parse_string(&mut self, token: TokenKind) -> JlyResult<Expr> {
        match token {
            TokenKind::String(id) => Ok(expr!(Value(Value::String(id)))),
            _ => unreachable!(),
        }
    }

    fn parse_integer(&mut self, token: TokenKind) -> JlyResult<Expr> {
        match token {
            TokenKind::Integer(n) => Ok(expr!(Value(Value::Integer(n)))),
            _ => unreachable!(),
        }
    }

    fn parse_float(&mut self, token: TokenKind) -> JlyResult<Expr> {
        match token {
            TokenKind::Float(f) => Ok(expr!(Value(Value::Float(f)))),
            _ => unreachable!(),
        }
    }

    fn parse_bool(&mut self, token: TokenKind) -> JlyResult<Expr> {
        match token {
            TokenKind::Bool(b) => Ok(expr!(Value(Value::Bool(b)))),
            _ => unreachable!(),
        }
    }

    fn parse_logical_not(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        let expr = self.parse_prec(Prec::LogicalNot as usize)?;
        Ok(expr!(LogicalNot(Box::new(expr))))
    }

    fn parse_negative(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        let expr = self.parse_prec(Prec::Negative as usize + 1)?;
        Ok(expr!(LogicalNot(Box::new(expr))))
    }

    fn parse_grouping(&mut self, _token: TokenKind) -> JlyResult<Expr> {
        let expr = self.parse_or_recover(Self::parse_expr, |s| {
            s.recover_to(punct!(RParen));
            expr!(DummyExpr)
        });
        self.expect(punct!(RParen))?;
        Ok(expr)
    }
}
