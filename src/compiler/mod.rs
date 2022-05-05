use passes::codegen::BytecodeEmitter;
use crate::runtime::chunk::Chunk;
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

impl<'sess> CompiledProgram<'sess> {
    #[allow(clippy::result_unit_err)]
    pub fn compile(
        source: &'sess Source,
        diagnostics: &mut ErrorReporter,
    ) -> Result<CompiledProgram<'sess>, ()> {
        let lexer = Lexer::new(source.cursor());
        let parser = Parser::new(lexer, diagnostics);

        let module_root = parser.parse();
        diagnostics.assert_ok()?;

        let mut chunk = Chunk::default();
        module_root.emit(&mut chunk);

        Ok(Self { source, chunk })
    }
}
