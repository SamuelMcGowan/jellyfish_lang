use crate::compiler::ast::*;
use crate::compiler::diagnostic::{Error, JlyResult};
use crate::compiler::passes::visit::Visitor;
use internment::Intern;

pub struct Binding {
    ident: Intern<String>,
    defined: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct VarResolved(usize);

impl VarResolved {
    pub fn byte(&self) -> u8 {
        self.0 as u8
    }
}

pub struct Resolver {
    vars: Vec<Binding>,
    scopes: Vec<usize>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            vars: vec![],
            scopes: vec![],
        }
    }

    fn start_scope(&mut self) {
        self.scopes.push(self.vars.len());
    }

    fn end_scope(&mut self) -> usize {
        let prev_len = self.scopes.pop().unwrap();
        self.vars.truncate(prev_len);
        prev_len
    }

    fn declare_var(&mut self, ident: Intern<String>) -> JlyResult<VarResolved> {
        let n = self.vars.len();

        self.vars.push(Binding {
            ident,
            defined: false,
        });

        if n > 0xff {
            return Err(Error::TooManyLocals);
        }

        Ok(VarResolved(n))
    }

    fn define_var(&mut self, var: VarResolved) {
        self.vars[var.0].defined = true;
    }

    fn resolve_var(&mut self, ident: Intern<String>) -> JlyResult<VarResolved> {
        self.vars
            .iter()
            .rposition(|binding| binding.ident == ident && binding.defined)
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
            ExprKind::Var(ident) => {
                let resolved = self.resolve_var(*ident)?;
                *expr = expr!(VarResolved(resolved));
            }
            ExprKind::VarResolved(_) => unreachable!(),

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

            ExprKind::LogicalNot(expr) | ExprKind::DebugPrint(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> JlyResult<()> {
        let var = self.declare_var(var_decl.ident)?;
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
}
