use crate::compiler::ast::*;
use crate::compiler::diagnostic::JlyResult;
use crate::runtime::chunk::{Chunk, Instr};
use crate::runtime::value::Value;

use super::visit::Visitor;

enum JumpKind {
    Jump,
    JumpNot,
}

impl JumpKind {
    pub fn instr(&self) -> Instr {
        match self {
            Self::Jump => Instr::JumpU32,
            Self::JumpNot => Instr::JumpNotU32,
        }
    }
}

// TODO: handle 64 bit addr case???
struct JumpSource(u32);

struct JumpDest(u32);

impl Chunk {
    pub fn emit_instr(&mut self, instr: Instr) {
        self.code.push(instr.into());
    }

    pub fn emit_constant(&mut self, value: Value) {
        let idx = self.constants.len();
        self.constants.push(value);

        if idx <= 0xff {
            self.emit_instr(Instr::LoadConstantU8);
            self.emit_u8(idx as u8);
        } else {
            self.emit_instr(Instr::LoadConstantU32);
            self.emit_u32(idx as u32);
        }
    }

    #[inline]
    fn emit_u8(&mut self, n: u8) {
        self.code.push((n as u8).into());
    }

    #[inline]
    fn emit_u32(&mut self, n: u32) {
        self.emit_u8((n & 0xff000000) as u8);
        self.emit_u8((n & 0x00ff0000) as u8);
        self.emit_u8((n & 0x0000ff00) as u8);
        self.emit_u8((n & 0x000000ff) as u8);
    }

    #[inline]
    fn set_u32(&mut self, idx: usize, n: u32) {
        self.code[idx].set_byte((n & 0xff000000) as u8);
        self.code[idx + 1].set_byte((n & 0x00ff0000) as u8);
        self.code[idx + 2].set_byte((n & 0x0000ff00) as u8);
        self.code[idx + 3].set_byte((n & 0x000000ff) as u8);
    }

    fn new_jump_source(&mut self, kind: JumpKind) -> JumpSource {
        self.emit_instr(kind.instr());

        let source = self.code.len() as u32;
        self.emit_u32(0);

        JumpSource(source)
    }

    fn new_jump_dest(&mut self) -> JumpDest {
        JumpDest(self.code.len() as u32)
    }

    fn jump_arrive(&mut self, source: JumpSource) {
        self.set_u32(source.0 as usize, self.code.len() as u32);
    }

    fn jump_depart(&mut self, dest: JumpDest, kind: JumpKind) {
        self.emit_instr(kind.instr());
        self.emit_u32(dest.0 as u32);
    }
}

#[derive(Default)]
pub struct CodeGenerator {
    chunk: Chunk,
}

impl CodeGenerator {
    pub fn chunk(self) -> Chunk {
        self.chunk
    }
}

impl Visitor for CodeGenerator {
    fn visit_module(&mut self, module: &mut Module) -> JlyResult<()> {
        for statement in &mut module.statements {
            self.visit_statement(statement)?;
        }
        self.chunk.emit_instr(Instr::Return);
        Ok(())
    }

    fn visit_block(&mut self, block: &mut Block) -> JlyResult<()> {
        for statement in &mut block.statements {
            self.visit_statement(statement)?;
        }
        for _ in 0..block.num_vars.unwrap() {
            self.chunk.emit_instr(Instr::Pop);
        }
        Ok(())
    }

    fn visit_statement(&mut self, statement: &mut Statement) -> JlyResult<()> {
        match statement {
            Statement::Expr(expr) => {
                self.visit_expr(expr)?;
                self.chunk.emit_instr(Instr::Pop);
            }
            Statement::Block(block) => self.visit_block(block)?,
            Statement::VarDecl(var_decl) => self.visit_var_decl(var_decl)?,
            Statement::If(if_statement) => self.visit_if_statement(if_statement)?,
            Statement::While(while_loop) => self.visit_while_loop(while_loop)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> JlyResult<()> {
        macro_rules! binary_op {
            ($a:ident $op:ident $b:ident) => {{
                self.visit_expr($a)?;
                self.visit_expr($b)?;
                self.chunk.emit_instr(Instr::$op);
            }};
        }

        match &mut expr.kind {
            ExprKind::Var(var) => self.visit_var(var)?,

            ExprKind::Value(value) => {
                self.chunk.emit_constant(value.clone());
            }

            ExprKind::LogicalOr(a, b) => binary_op!(a OrBool b),
            ExprKind::LogicalAnd(a, b) => binary_op!(a AndBool b),
            ExprKind::LogicalNot(a) => {
                self.visit_expr(a)?;
                self.chunk.emit_instr(Instr::NotBool);
            }

            ExprKind::Equal(a, b) => binary_op!(a Equal b),
            ExprKind::NotEqual(a, b) => {
                binary_op!(a Equal b);
                self.chunk.emit_instr(Instr::NotBool);
            }
            ExprKind::LT(a, b) => binary_op!(a LT b),
            ExprKind::GT(a, b) => binary_op!(b LT a),
            ExprKind::LTEqual(a, b) => binary_op!(a LTEqual b),
            ExprKind::GTEqual(a, b) => binary_op!(b LTEqual a),

            ExprKind::Add(a, b) => binary_op!(a AddInt b),
            ExprKind::Sub(a, b) => binary_op!(a SubInt b),
            ExprKind::Mul(a, b) => binary_op!(a MulInt b),
            ExprKind::Div(a, b) => binary_op!(a DivInt b),
            ExprKind::Mod(a, b) => binary_op!(a ModInt b),
            ExprKind::Pow(a, b) => binary_op!(a PowInt b),
            ExprKind::Neg(expr) => {
                self.visit_expr(expr)?;
                self.chunk.emit_instr(Instr::NegInt);
            }

            ExprKind::Assignment(lhs, rhs) => {
                self.visit_expr(rhs)?;
                self.chunk.emit_instr(Instr::StoreLocal);
                self.chunk.emit_u8(lhs.resolved.unwrap().byte());
            }

            ExprKind::DebugPrint(expr) => {
                self.visit_expr(expr)?;
                self.chunk.emit_instr(Instr::DebugPrint);
                self.chunk.emit_instr(Instr::LoadUnit);
            }

            ExprKind::DummyExpr => unreachable!(),
        }

        Ok(())
    }

    fn visit_var(&mut self, var: &mut Var) -> JlyResult<()> {
        self.chunk.emit_instr(Instr::LoadLocal);
        self.chunk.emit_u8(var.resolved.unwrap().byte());
        Ok(())
    }

    fn visit_var_decl(&mut self, var_decl: &mut VarDecl) -> JlyResult<()> {
        self.visit_expr(&mut var_decl.value)
    }

    fn visit_if_statement(&mut self, if_statement: &mut IfStatement) -> JlyResult<()> {
        self.visit_expr(&mut if_statement.condition)?;

        if let Some(else_) = &mut if_statement.else_ {
            let else_jump = self.chunk.new_jump_source(JumpKind::JumpNot);

            self.visit_block(&mut if_statement.then)?;
            let end_jump = self.chunk.new_jump_source(JumpKind::Jump);

            self.chunk.jump_arrive(else_jump);
            self.visit_statement(else_)?;

            self.chunk.jump_arrive(end_jump);
        } else {
            let end_jump = self.chunk.new_jump_source(JumpKind::JumpNot);
            self.visit_block(&mut if_statement.then)?;
            self.chunk.jump_arrive(end_jump);
        }

        Ok(())
    }

    fn visit_while_loop(&mut self, while_loop: &mut WhileLoop) -> JlyResult<()> {
        let top_jump = self.chunk.new_jump_dest();

        self.visit_expr(&mut while_loop.condition)?;
        let end_jump = self.chunk.new_jump_source(JumpKind::JumpNot);

        self.visit_block(&mut while_loop.body)?;
        self.chunk.jump_depart(top_jump, JumpKind::Jump);

        self.chunk.jump_arrive(end_jump);

        Ok(())
    }
}
