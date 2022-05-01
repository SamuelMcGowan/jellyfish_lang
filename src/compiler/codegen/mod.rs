use crate::compiler::ast::{Expr, Module, Statement};
use crate::runtime::chunk::{Chunk, Instr, Opcode};
use crate::runtime::value::Value;

impl Chunk {
    pub fn emit_instr(&mut self, instr: Instr) {
        self.code.push(Opcode { instr });
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

    fn emit_u8(&mut self, n: u8) {
        self.code.push(Opcode { byte: n as u8 });
    }

    fn emit_u32(&mut self, n: u32) {
        self.emit_u8((n & 0xff000000) as u8);
        self.emit_u8((n & 0x00ff0000) as u8);
        self.emit_u8((n & 0x0000ff00) as u8);
        self.emit_u8((n & 0x000000ff) as u8);
    }
}

pub trait BytecodeEmitter<'a> {
    fn emit(&self, chunk: &mut Chunk);
}

impl<'a> BytecodeEmitter<'a> for Module {
    fn emit(&self, chunk: &mut Chunk) {
        for stmt in &self.statements {
            stmt.emit(chunk);
        }

        // TODO: separate exit instruction?
        chunk.emit_instr(Instr::Return);
    }
}

impl<'a> BytecodeEmitter<'a> for Statement {
    fn emit(&self, chunk: &mut Chunk) {
        match self {
            Self::DebugPrint(expr) => {
                expr.emit(chunk);
                chunk.emit_instr(Instr::DebugPrint);
            }
            Self::ExprStmt(expr) => {
                expr.emit(chunk);
                chunk.emit_instr(Instr::Pop);
            }
        }
    }
}

impl<'a> BytecodeEmitter<'a> for Expr {
    fn emit(&self, chunk: &mut Chunk) {
        macro_rules! binary_op {
            ($a:ident $op:ident $b:ident) => {{
                $a.emit(chunk);
                $b.emit(chunk);
                chunk.emit_instr(Instr::$op);
            }};
        }

        match self {
            Self::Value(value) => {
                chunk.emit_constant(value.clone());
            }

            Self::LogicalOr(a, b) => binary_op!(a OrBool b),
            Self::LogicalAnd(a, b) => binary_op!(a AndBool b),
            Self::LogicalNot(a) => {
                a.emit(chunk);
                chunk.emit_instr(Instr::NotBool);
            }

            Self::Equal(a, b) => binary_op!(a Equal b),
            Self::NotEqual(a, b) => {
                binary_op!(a Equal b);
                chunk.emit_instr(Instr::NotBool);
            }
            Self::LT(a, b) => binary_op!(a LT b),
            Self::GT(a, b) => binary_op!(b LT a),
            Self::LTEqual(a, b) => binary_op!(a LTEqual b),
            Self::GTEqual(a, b) => binary_op!(b LTEqual a),

            Self::Add(a, b) => binary_op!(a AddInt b),
            Self::Sub(a, b) => binary_op!(a SubInt b),
            Self::Mul(a, b) => binary_op!(a MulInt b),
            Self::Div(a, b) => binary_op!(a DivInt b),
            Self::Mod(a, b) => binary_op!(a Mod b),
            Self::Pow(a, b) => binary_op!(a Pow b),

            _ => todo!("can't emit bytecode for expression"),
        }
    }
}
