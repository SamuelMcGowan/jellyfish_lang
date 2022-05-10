use crate::compiler::ast::Module;
use crate::compiler::diagnostic::JlyResult;
use crate::compiler::passes::codegen::CodeGenerator;
use crate::compiler::passes::resolve::Resolver;
use crate::compiler::passes::visit::Visitor;
use crate::runtime::chunk::Chunk;

pub mod codegen;
pub mod resolve;
pub mod visit;

pub fn run_passes(module: &mut Module) -> JlyResult<Chunk> {
    let mut resolver = Resolver::new();
    resolver.visit_module(module)?;

    let mut codegen = CodeGenerator::default();
    codegen.visit_module(module)?;

    Ok(codegen.chunk())
}
