use crate::compiler::passes::codegen::CodeGenerator;
use crate::compiler::passes::resolve::Resolver;
use crate::compiler::passes::visit::Visitor;
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

        let mut module_root = parser.parse();
        diagnostics.assert_ok()?;

        let mut resolver = Resolver::new();
        if let Err(e) = resolver.visit_module(&mut module_root) {
            diagnostics.report(e.report());
            return Err(());
        }

        let mut code_gen = CodeGenerator::default();
        if let Err(e) = code_gen.visit_module(&mut module_root) {
            diagnostics.report(e.report());
            return Err(());
        }

        let chunk = code_gen.chunk();

        Ok(Self { source, chunk })
    }
}
