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

        Ok(Block {
            statements,
            num_vars: None,
        })
    }

    pub fn parse_statement(&mut self) -> Statement {
        self.parse_or_recover(Self::parse_statement_inner, |s, span| {
            s.recover_past(punct!(Semicolon));
            Statement::Expr(expr!(DummyExpr, span))
        })
    }

    fn parse_statement_inner(&mut self) -> JlyResult<Statement> {
        match self.cursor.peek().kind {
            kwd!(Let) => Ok(Statement::VarDecl(self.parse_var_decl()?)),
            kwd!(If) => Ok(Statement::If(self.parse_if_statement()?)),
            kwd!(While) => Ok(Statement::While(self.parse_while_loop()?)),
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
        let let_token = self.expect(kwd!(Let))?;

        let ident_token = self.cursor.next();
        let ident = match ident_token.kind {
            TokenKind::Ident(ident) => ident,
            _ => return Err(Error::ExpectedIdent(ident_token)),
        };

        self.expect(punct!(Equal))?;

        let value = Box::new(self.parse_expr()?);

        let semicolon_token = self.expect(punct!(Semicolon))?;

        let span = let_token.span.join(semicolon_token.span);

        Ok(VarDecl { ident, value, span })
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

    fn parse_while_loop(&mut self) -> JlyResult<WhileLoop> {
        self.expect(kwd!(While))?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(WhileLoop { condition, body })
    }
}
