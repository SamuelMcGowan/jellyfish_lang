use super::*;

impl<'sess> Parser<'sess> {
    pub fn parse_statement(&mut self) -> Statement {
        self.parse_or_recover(Self::parse_statement_inner, |s| {
            s.recover_past(punct!(Semicolon));
            Statement::new(expr!(DummyExpr))
        })
    }

    pub fn parse_statement_inner(&mut self) -> JlyResult<Statement> {
        let statement = match self.cursor.peek().kind {
            kwd!(If) => {
                self.cursor.next();
                Statement::new(expr!(boxed IfStatement(self.parse_if_statement()?)))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(punct!(Semicolon))?;
                Statement::new(expr)
            }
        };
        Ok(statement)
    }

    pub fn parse_if_statement(&mut self) -> JlyResult<IfStatement> {
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

        Ok(IfStatement {
            condition,
            then,
            else_,
        })
    }

    fn parse_if_arm(&mut self) -> JlyResult<Expr> {
        let token = self.cursor.peek();
        if token.kind == punct!(LBrace) || token.kind == punct!(LParen) {
            self.parse_expr()
        } else {
            Err(Error::ExpectedIfArm(self.cursor.next()))
        }
    }

    pub fn parse_block(&mut self) -> JlyResult<Vec<Statement>> {
        let mut statements = vec![];

        while !(self.cursor.eof() || self.cursor.matches(punct!(RBrace))) {
            statements.push(self.parse_statement());
        }
        self.expect(punct!(RBrace))?;

        Ok(statements)
    }
}
