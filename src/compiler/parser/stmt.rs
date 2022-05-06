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

    pub fn parse_statement_inner(&mut self) -> JlyResult<Statement> {
        let expr = self.parse_expr()?;

        if expr.kind.requires_semicolon() {
            self.expect(punct!(Semicolon))?;
        } else {
            self.cursor.eat(punct!(Semicolon));
        }

        Ok(Statement::Expr(expr))
    }
}
