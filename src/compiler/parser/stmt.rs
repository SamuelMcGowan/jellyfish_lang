use super::*;

impl<'sess> Parser<'sess> {
    pub fn parse_block(&mut self) -> JlyResult<Block> {
        let mut statements = vec![];

        self.expect(punct!(LBrace))?;

        while !(self.cursor.eof() || self.cursor.matches(punct!(RBrace))) {
            statements.push(self.parse_statement());
        }

        // no use delegating this to the caller, deal with it here.
        if let Err(e) = self.expect(punct!(RBrace)) {
            self.diagnostics.report(e.report());
            self.recover_past(punct!(RBrace));
        }

        Ok(Block { statements })
    }

    pub fn parse_statement(&mut self) -> Statement {
        self.parse_or_recover(Self::parse_statement_inner, |s| {
            s.recover_past(punct!(Semicolon));
            Statement::Expr(expr!(DummyExpr))
        })
    }

    fn parse_statement_inner(&mut self) -> JlyResult<Statement> {
        match self.cursor.peek().kind {
            kwd!(Let) => Ok(Statement::VarDecl(self.parse_var_decl()?)),
            kwd!(If) => Ok(Statement::If(self.parse_if_statement()?)),
            punct!(LBrace) => {
                let block = self.parse_block()?;
                Ok(Statement::Block(block))
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(punct!(Semicolon))?;

                Ok(Statement::Expr(expr))
            }
        }
    }

    fn parse_var_decl(&mut self) -> JlyResult<VarDecl> {
        self.expect(kwd!(Let))?;

        let token = self.cursor.next();
        let ident = match token.kind {
            TokenKind::Ident(ident) => ident,
            _ => return Err(Error::ExpectedIdent(token)),
        };

        let value = if self.cursor.eat(punct!(Equal)) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };

        self.expect(punct!(Semicolon))?;

        Ok(VarDecl { ident, value })
    }

    fn parse_if_statement(&mut self) -> JlyResult<IfStatement> {
        self.expect(kwd!(If))?;

        let condition = self.parse_expr()?;

        let then = self.parse_block()?;

        let else_ = if self.cursor.eat(kwd!(Else)) {
            let stmt = if self.cursor.eat(kwd!(If)) {
                Statement::If(self.parse_if_statement()?)
            } else {
                Statement::Block(self.parse_block()?)
            };
            Some(Box::new(stmt))
        } else {
            None
        };

        Ok(IfStatement {
            condition,
            then,
            else_,
        })
    }
}
