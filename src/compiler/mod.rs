use crate::compiler::passes::run_passes;
use crate::runtime::CompiledProgram;
use crate::Source;

use self::diagnostic::ErrorReporter;
use self::lexer::Lexer;
use self::parser::Parser;

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod passes;
pub mod symbol;

impl<'sess> CompiledProgram<'sess> {
    #[allow(clippy::result_unit_err)]
    pub fn compile(
        source: &'sess Source,
        diagnostics: &mut ErrorReporter,
    ) -> Result<CompiledProgram<'sess>, ()> {
        let lexer = Lexer::new(source.cursor());
        let parser = Parser::new(lexer, diagnostics);

        let mut module = parser.parse();
        diagnostics.assert_ok()?;

        let chunk = match run_passes(&mut module) {
            Ok(chunk) => chunk,
            Err(err) => {
                diagnostics.report(err.report());
                return Err(());
            }
        };

        Ok(Self { source, chunk })
    }
}
