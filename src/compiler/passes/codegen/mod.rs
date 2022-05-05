use crate::compiler::ast::{Expr, IfStatement, Module, Statement};
use crate::runtime::chunk::{Chunk, Instr, Opcode};
use crate::runtime::value::Value;

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
struct Jump(u32);

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

    fn new_jump(&mut self, kind: JumpKind) -> Jump {
        self.emit_instr(kind.instr());

        let source = self.code.len() as u32;
        self.emit_u32(0);

        Jump(source)
    }

    fn jump_arrive(&mut self, source: Jump) {
        let from = source.0 as usize;
        let dest = self.code.len();
        self.code[from].set_byte((dest & 0xff000000) as u8);
        self.code[from + 1].set_byte((dest & 0x00ff0000) as u8);
        self.code[from + 2].set_byte((dest & 0x0000ff00) as u8);
        self.code[from + 3].set_byte((dest & 0x000000ff) as u8);
    }
}

pub trait BytecodeEmitter {
    fn emit(&self, chunk: &mut Chunk);
}

impl BytecodeEmitter for Module {
    fn emit(&self, chunk: &mut Chunk) {
        for stmt in &self.statements {
            stmt.emit(chunk);
        }

        // TODO: separate exit instruction?
        chunk.emit_instr(Instr::Return);
    }
}

impl BytecodeEmitter for Statement {
    fn emit(&self, chunk: &mut Chunk) {
        match self {
            Self::If(if_statement) => if_statement.emit(chunk),
            Self::DebugPrint(expr) => {
                expr.emit(chunk);
                chunk.emit_instr(Instr::DebugPrint);
            }
            Self::ExprStmt(expr) => {
                expr.emit(chunk);
                chunk.emit_instr(Instr::Pop);
            }
            Self::DummyStmt => unreachable!(),
        }
    }
}

impl BytecodeEmitter for Expr {
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

            Self::Block(statements) => {
                for statement in statements {
                    statement.emit(chunk);
                }
                // TODO: remember to pop values here
            }
            Self::IfStatement(if_statement) => if_statement.emit(chunk),

            Self::DummyExpr => unreachable!(),

            other => todo!("can't emit bytecode for expression {}", other),
        }
    }
}

impl BytecodeEmitter for IfStatement {
    fn emit(&self, chunk: &mut Chunk) {
        self.condition.emit(chunk);

        if let Some(else_) = &self.else_ {
            let else_jump = chunk.new_jump(JumpKind::JumpNot);

            self.then.emit(chunk);
            let end_jump = chunk.new_jump(JumpKind::Jump);

            chunk.jump_arrive(else_jump);
            else_.emit(chunk);

            chunk.jump_arrive(end_jump);
        } else {
            let end_jump = chunk.new_jump(JumpKind::JumpNot);
            self.then.emit(chunk);
            chunk.jump_arrive(end_jump);
        }
    }
}