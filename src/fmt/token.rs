use crate::compiler::lexer::token::Token;

use super::*;

impl<'a> DisplayWithContext<'a> for Token {
    type Context = Source;

    fn fmt(&self, f: &mut std::fmt::Formatter, context: &Self::Context) -> std::fmt::Result {
        let line_col = context.line_col(self.span.start);
        write!(
            f,
            "Token({}:{}, {:?})",
            line_col.line,
            line_col.col,
            self.kind,
        )
    }
}
