use crate::compiler::ast::*;
use crate::compiler::diagnostic::JlyResult;

pub trait Visitor {
    fn visit_module(&mut self, module: &mut Module) -> JlyResult<()> {
        for statement in &mut module.statements {
            self.visit_statement(statement)?;
        }
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> JlyResult<()> {
        for statement in &mut block.statements {
            self.visit_statement(statement)?;
        }
        Ok(())
    }

    fn visit_statement(&mut self, statement: &mut Statement) -> JlyResult<()> {
        match statement {
            Statement::Expr(expr) => self.visit_expr(expr)?,
            Statement::VarDecl(var_decl) => self.visit_var_decl(var_decl)?,
            Statement::Block(block) => self.visit_block(block)?,
            Statement::If(if_statement) => self.visit_if_statement(if_statement)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> JlyResult<()>;

    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> JlyResult<()>;

    fn visit_if_statement(&mut self, if_statement: &mut IfStatement) -> JlyResult<()>;
}
