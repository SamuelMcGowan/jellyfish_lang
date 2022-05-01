use crate::compiler::codegen::BytecodeEmitter;
use crate::runtime::chunk::Chunk;
use crate::Source;

use self::diagnostic::Diagnostics;
use self::lexer::Lexer;
use self::parser::Parser;

pub mod ast;
pub mod codegen;
pub mod diagnostic;
pub mod lexer;
pub mod parser;

pub struct CompiledModule {
    pub source: Source,
    pub chunk: Chunk,
}

impl CompiledModule {
    #[allow(clippy::result_unit_err)]
    pub fn compile(source: Source, diagnostics: &mut Diagnostics) -> Result<CompiledModule, ()> {
        let lexer = Lexer::new(source.cursor());
        let parser = Parser::new(lexer, diagnostics);

        let module_root = parser.parse();
        diagnostics.assert_ok()?;

        let mut chunk = Chunk::default();
        module_root.emit(&mut chunk);

        Ok(Self { source, chunk })
    }
}
