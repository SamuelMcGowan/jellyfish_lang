use internment::Intern;

use crate::compiler::ast::*;
use crate::compiler::diagnostic::{Error, JlyResult};
use crate::compiler::passes::visit::Visitor;
use crate::compiler::symbol::{Symbol, SymbolTable};
use crate::source::Span;

#[derive(Default, Debug, Clone, Copy)]
pub struct VarResolved(usize);

impl VarResolved {
    pub fn byte(&self) -> u8 {
        self.0 as u8
    }
}

pub struct VarEntry {
    ident: Intern<String>,
    defined: bool,
}

pub struct Resolver {
    vars: SymbolTable<VarEntry>,
    stack: Vec<Symbol<VarEntry>>,
    scopes: Vec<usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            vars: SymbolTable::new(),
            stack: vec![],
            scopes: vec![],
        }
    }

    fn start_scope(&mut self) {
        self.scopes.push(self.stack.len());
    }

    fn end_scope(&mut self) -> usize {
        let prev_len = self.scopes.pop().unwrap();
        let scope_size = self.stack.len() - prev_len;

        self.stack.truncate(prev_len);

        scope_size
    }

    fn declare_var(
        &mut self,
        ident: Intern<String>,
        ident_span: Span,
    ) -> JlyResult<Symbol<VarEntry>> {
        let local_number = self.stack.len();

        let entry = VarEntry {
            ident,
            defined: false,
        };
        let symbol = self.vars.add_entry(entry);

        self.stack.push(symbol);

        if local_number > 0xff {
            return Err(Error::TooManyLocals(ident_span));
        }

        Ok(symbol)
    }

    fn define_var(&mut self, symbol: Symbol<VarEntry>) {
        self.vars.get_mut(symbol).defined = true;
    }

    fn resolve_var(&mut self, ident: Intern<String>) -> JlyResult<VarResolved> {
        self.stack
            .iter()
            .rposition(|symbol| self.vars.get(*symbol).ident == ident)
            .map(VarResolved)
            .ok_or(Error::UnresolvedVariable(ident))
    }
}

impl Visitor for Resolver {
    fn visit_block(&mut self, block: &mut Block) -> JlyResult<()> {
        self.start_scope();

        for statement in &mut block.statements {
            self.visit_statement(statement)?;
        }

        block.num_vars = Some(self.end_scope());

        Ok(())
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> JlyResult<()> {
        match &mut expr.kind {
            ExprKind::Var(var) => self.visit_var(var)?,
            ExprKind::Value(_) | ExprKind::DummyExpr => {}

            ExprKind::LogicalOr(lhs, rhs)
            | ExprKind::LogicalAnd(lhs, rhs)
            | ExprKind::Equal(lhs, rhs)
            | ExprKind::NotEqual(lhs, rhs)
            | ExprKind::LT(lhs, rhs)
            | ExprKind::GT(lhs, rhs)
            | ExprKind::LTEqual(lhs, rhs)
            | ExprKind::GTEqual(lhs, rhs)
            | ExprKind::Add(lhs, rhs)
            | ExprKind::Sub(lhs, rhs)
            | ExprKind::Mul(lhs, rhs)
            | ExprKind::Div(lhs, rhs)
            | ExprKind::Mod(lhs, rhs)
            | ExprKind::Pow(lhs, rhs) => {
                self.visit_expr(lhs)?;
                self.visit_expr(rhs)?;
            }

            ExprKind::Neg(expr) => self.visit_expr(expr)?,

            ExprKind::Assignment(lhs, rhs) => {
                self.visit_var(lhs)?;
                self.visit_expr(rhs)?;
            }

            ExprKind::LogicalNot(expr) | ExprKind::DebugPrint(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_var(&mut self, var: &mut Var) -> JlyResult<()> {
        var.resolved = Some(self.resolve_var(var.ident)?);
        Ok(())
    }

    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> JlyResult<()> {
        let var = self.declare_var(var_decl.ident, var_decl.span)?;
        self.visit_expr(&mut var_decl.value)?;
        self.define_var(var);
        Ok(())
    }

    fn visit_if_statement(&mut self, if_statement: &mut IfStatement) -> JlyResult<()> {
        self.visit_expr(&mut if_statement.condition)?;
        self.visit_block(&mut if_statement.then)?;
        if let Some(else_) = &mut if_statement.else_ {
            self.visit_statement(else_)?;
        }
        Ok(())
    }

    fn visit_while_loop(&mut self, while_loop: &mut WhileLoop) -> JlyResult<()> {
        self.visit_expr(&mut while_loop.condition)?;
        self.visit_block(&mut while_loop.body)
    }
}
