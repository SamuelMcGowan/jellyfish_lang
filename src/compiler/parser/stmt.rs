use super::*;

impl ExprKind {
    pub fn requires_semicolon(&self) -> bool {
        !matches!(self, Self::Block(_) | Self::If(_))
    }
}

impl<'sess> Parser<'sess> {
    pub fn parse_statement(&mut self) -> Statement {
        self.parse_or_recover(Self::parse_statement_inner, |s| {
            s.recover_past(punct!(Semicolon));
            Statement::Expr(expr!(DummyExpr))
        })
    }

    fn parse_statement_inner(&mut self) -> JlyResult<Statement> {
        match self.cursor.peek().kind {
            kwd!(Let) => Ok(Statement::VarDecl(self.parse_var_decl()?)),
            _ => {
                let expr = self.parse_expr()?;

                if expr.kind.requires_semicolon() {
                    self.expect(punct!(Semicolon))?;
                } else {
                    self.cursor.eat(punct!(Semicolon));
                }

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
}
