use super::*;

impl<'sess> Parser<'sess> {
    pub fn parse_stmt(&mut self) -> ParseResult<Statement> {
        let statement = match self.cursor.peek().kind {
            kwd!(DebugPrint) => {
                self.cursor.next();
                let expr = self.parse_expr()?;
                self.expect(punct!(Semicolon))?;
                Statement::DebugPrint(expr)
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(punct!(Semicolon))?;
                Statement::ExprStmt(expr)
            }
        };
        Ok(statement)
    }
}
